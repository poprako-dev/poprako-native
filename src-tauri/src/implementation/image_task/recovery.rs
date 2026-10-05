use std::sync::Arc;

use crate::data::image_import::ImageTaskPhase;
use crate::harness::Harness;
use crate::implementation::image_task::PublishedImage;
use crate::implementation::resource::recovery::{discard_stale_staging, original_candidates};
use crate::result::{AppError, AppResult};
use crate::usecase::page;

/// # Errors
/// Cleanup requires successful migration, no active task, and readable database references.
pub async fn recover(harness: Arc<Harness>) -> AppResult<()> {
    let permit = Arc::clone(&harness.resource_task)
        .try_acquire_owned()
        .map_err(|_| AppError::Busy)?;

    let coordinator = harness.database().await?;

    let _barrier = coordinator.recovery_barrier().await;

    if coordinator.is_halted() {
        return Err(AppError::RecoveryRequired);
    }

    let worker = Arc::clone(&harness);

    let candidates = tokio::task::spawn_blocking(move || original_candidates(&worker.resource))
        .await
        .map_err(|_| AppError::RecoveryRequired)?
        .map_err(|_| AppError::Storage)?;

    let mut unused = Vec::new();

    // Finish all reference reads before deleting anything: unreadable is never unreferenced.
    for candidate in candidates {
        let referenced = match page::get_page(
            coordinator,
            &candidate.comic_id.to_string(),
            &candidate.page_id.to_string(),
        )
        .await
        {
            Ok(page) => page.image.reference == candidate.reference,
            Err(AppError::NotFound) => false,
            Err(error) => return Err(error),
        };

        if !referenced && !harness.resource_registry.protects(&candidate.reference)? {
            unused.push(candidate);
        }
    }

    let worker = Arc::clone(&harness);

    tokio::task::spawn_blocking(move || {
        for candidate in unused {
            worker
                .resource
                .remove_unreferenced(&candidate.reference, candidate.comic_id, candidate.page_id)
                .map_err(|_| AppError::Storage)?;
        }

        discard_stale_staging(&worker.resource).map_err(|_| AppError::Storage)
    })
    .await
    .map_err(|_| AppError::RecoveryRequired)??;

    drop(permit);

    Ok(())
}

/// # Errors
/// Deletes only this known old immutable reference after successful commit and reread.
pub async fn cleanup_old(
    harness: &Harness,
    comic_id: &str,
    page_id: &str,
    reference: &str,
) -> AppResult<()> {
    let coordinator = harness.database().await?;

    let _barrier = coordinator.recovery_barrier().await;

    if coordinator.is_halted() {
        return Err(AppError::RecoveryRequired);
    }

    let referenced = match page::get_page(coordinator, comic_id, page_id).await {
        Ok(current) => current.image.reference == reference,
        Err(AppError::NotFound) => false,
        Err(error) => return Err(error),
    };

    if referenced || harness.resource_registry.protects(reference)? {
        return Err(AppError::Busy);
    }

    let comic_id = uuid::Uuid::parse_str(comic_id).map_err(|_| AppError::InvalidInput)?;

    let page_id = uuid::Uuid::parse_str(page_id).map_err(|_| AppError::InvalidInput)?;

    harness
        .resource
        .remove_unreferenced(reference, comic_id, page_id)
        .map_err(|_| AppError::Storage)
}

async fn retry_images(
    harness: &Harness,
    images: Vec<PublishedImage>,
) -> (Vec<PublishedImage>, Option<AppError>) {
    let mut pending = Vec::new();

    let mut failure = None;

    for image in images {
        if let Err(error) = cleanup_old(
            harness,
            &image.comic_id.to_string(),
            &image.page_id.to_string(),
            &image.reference,
        )
        .await
        {
            pending.push(image);

            failure.get_or_insert(error);
        }
    }

    (pending, failure)
}

/// # Errors
/// Retains every failed or protected resource; a retry never repeats the database write.
pub async fn retry(harness: &Harness, task_id: &str) -> AppResult<()> {
    let (files, published, obsolete) = harness.image_task.update(task_id, |task| {
        if !matches!(
            task.status.phase,
            ImageTaskPhase::Completed | ImageTaskPhase::Cancelled | ImageTaskPhase::Failed
        ) {
            return Err(AppError::Busy);
        }

        Ok((
            task.files.clone(),
            task.published.clone(),
            task.obsolete.clone(),
        ))
    })?;

    let mut pending = Vec::new();

    let mut failure = None;

    for file in files {
        if harness.resource.discard_prepared(&file.image).is_err() {
            pending.push(file);

            failure.get_or_insert(AppError::Storage);
        }
    }

    let (published, published_error) = retry_images(harness, published).await;

    let (obsolete, obsolete_error) = retry_images(harness, obsolete).await;

    let failure = failure.or(published_error).or(obsolete_error);

    harness.image_task.update(task_id, |task| {
        task.files = pending;

        task.published = published;

        task.obsolete = obsolete;

        task.status.cleanup_pending = failure.is_some();

        if task.status.phase == ImageTaskPhase::Completed {
            task.status.message = match failure {
                Some(_) => "图片已保存，旧图片或暂存资源待清理；下次导入时将重试".to_owned(),
                None => "图片已保存".to_owned(),
            };
        }

        Ok(())
    })?;

    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
