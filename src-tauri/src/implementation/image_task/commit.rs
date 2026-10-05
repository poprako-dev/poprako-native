use std::sync::Arc;

use uuid::Uuid;

use crate::data::image_import::{ImageImportTarget, ImageTaskPhase};
use crate::data::page::NewPage;
use crate::harness::Harness;
use crate::implementation::image_task::prepare::cleanup;
use crate::implementation::image_task::recovery::retry;
use crate::implementation::image_task::{PreparedFile, PublishedImage};
use crate::model::page::Page;
use crate::result::{AppError, AppResult};
use crate::usecase::page;
use crate::value::image::{Image, ImageFormat};

fn format(extension: &str) -> AppResult<ImageFormat> {
    match extension {
        "jpg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "webp" => Ok(ImageFormat::Webp),
        "bmp" => Ok(ImageFormat::Bmp),
        _ => Err(AppError::InvalidInput),
    }
}

fn publish(
    harness: &Harness,
    task_id: &str,
    target: &ImageImportTarget,
    files: Vec<PreparedFile>,
) -> AppResult<Vec<NewPage>> {
    let comic_id = Uuid::parse_str(target.comic_id()).map_err(|_| AppError::InvalidInput)?;

    let mut pages = Vec::new();

    for file in files {
        if harness.image_task.status(task_id)?.phase == ImageTaskPhase::Stopping {
            return Err(AppError::Busy);
        }

        let page_id = match target {
            ImageImportTarget::Append { .. } => Uuid::new_v4(),
            ImageImportTarget::Replace { baseline, .. } => {
                Uuid::parse_str(&baseline.id).map_err(|_| AppError::InvalidInput)?
            }
        };

        let reference = harness
            .resource
            .publish(&file.image, comic_id, page_id)
            .map_err(|_| AppError::Storage)?;

        harness.image_task.update(task_id, |task| {
            task.published.push(PublishedImage {
                reference: reference.clone(),
                comic_id,
                page_id,
            });

            Ok(())
        })?;

        pages.push(NewPage {
            id: page_id.to_string(),
            image: Image {
                reference,
                original_name: file.original_name,
                format: format(&file.image.extension)?,
                width: file.image.width,
                height: file.image.height,
            },
        });
    }

    Ok(pages)
}

fn rollback_files(harness: &Harness, task_id: &str, error: &AppError) -> AppResult<()> {
    let published = harness
        .image_task
        .update(task_id, |task| Ok(std::mem::take(&mut task.published)))?;

    let mut pending = Vec::new();

    for image in published {
        if harness
            .resource
            .remove_unreferenced(&image.reference, image.comic_id, image.page_id)
            .is_err()
        {
            pending.push(image);
        }
    }

    let phase = match harness.image_task.status(task_id)?.phase {
        ImageTaskPhase::Stopping => ImageTaskPhase::Cancelled,
        _ => ImageTaskPhase::Failed,
    };

    cleanup(harness, task_id, phase, error.to_string())?;

    harness.image_task.update(task_id, |task| {
        task.status.cleanup_pending |= !pending.is_empty();

        task.published = pending;

        task.permit = None;

        Ok(())
    })
}

async fn apply(
    harness: &Harness,
    target: &ImageImportTarget,
    pages: &[NewPage],
    clear_units: bool,
) -> AppResult<Vec<Page>> {
    match target {
        ImageImportTarget::Append { comic_id, baseline } => {
            page::append_pages(harness.database().await?, comic_id, baseline, pages).await
        }
        ImageImportTarget::Replace { comic_id, baseline } => {
            let page = pages
                .first()
                .filter(|_| pages.len() == 1)
                .ok_or(AppError::InvalidInput)?;

            Ok(vec![
                page::replace_page_image(
                    harness.database().await?,
                    comic_id,
                    baseline.clone(),
                    page.image.clone(),
                    clear_units,
                )
                .await?,
            ])
        }
    }
}

/// # Errors
/// Preserves all published originals on uncertain commits; never retries the mutation.
pub async fn confirm(
    harness: Arc<Harness>,
    task_id: String,
    clear_units: bool,
) -> AppResult<Vec<Page>> {
    let (target, files) = harness.image_task.update(&task_id, |task| {
        if task.status.phase != ImageTaskPhase::Ready
            || task.files.is_empty()
            || clear_units && !matches!(task.target, ImageImportTarget::Replace { .. })
        {
            return Err(AppError::InvalidInput);
        }

        task.status.phase = ImageTaskPhase::Publishing;

        Ok((task.target.clone(), task.files.clone()))
    })?;

    let validation = async {
        crate::implementation::image_task::prepare::validate_target(&harness, &target).await?;

        Ok::<(), AppError>(())
    }
    .await;

    if let Err(error) = validation {
        rollback_files(&harness, &task_id, &error)?;

        return Err(error);
    }

    let worker = Arc::clone(&harness);

    let worker_id = task_id.clone();

    let worker_target = target.clone();

    let published =
        tokio::task::spawn_blocking(move || publish(&worker, &worker_id, &worker_target, files))
            .await
            .map_err(|_| AppError::RecoveryRequired)
            .and_then(|value| value);

    let pages = match published {
        Ok(pages) => pages,
        Err(error) => {
            rollback_files(&harness, &task_id, &error)?;

            return Err(error);
        }
    };

    let committing = harness.image_task.update(&task_id, |task| {
        if task.status.phase == ImageTaskPhase::Stopping {
            return Err(AppError::Busy);
        }

        task.status.phase = ImageTaskPhase::Committing;

        Ok(())
    });

    if let Err(error) = committing {
        rollback_files(&harness, &task_id, &error)?;

        return Err(error);
    }

    match apply(&harness, &target, &pages, clear_units).await {
        Ok(pages) => {
            cleanup(
                &harness,
                &task_id,
                ImageTaskPhase::Completed,
                "图片已保存".to_owned(),
            )?;

            harness.image_task.update(&task_id, |task| {
                if let ImageImportTarget::Replace { comic_id, baseline } = &target {
                    task.obsolete.push(PublishedImage {
                        reference: baseline.image_reference.clone(),
                        comic_id: Uuid::parse_str(comic_id).map_err(|_| AppError::InvalidInput)?,
                        page_id: Uuid::parse_str(&baseline.id)
                            .map_err(|_| AppError::InvalidInput)?,
                    });
                }

                task.published.clear();

                task.status.cleanup_pending |= !task.obsolete.is_empty();

                Ok(())
            })?;

            // The content is committed; cleanup failure remains a warning on this task.
            let _cleanup_result = retry(&harness, &task_id).await;

            harness.image_task.update(&task_id, |task| {
                task.permit = None;

                Ok(())
            })?;

            Ok(pages)
        }
        Err(error @ (AppError::CommitUncertain | AppError::RecoveryRequired)) => {
            harness.image_task.update(&task_id, |task| {
                task.status.phase = ImageTaskPhase::Uncertain;

                task.status.message = error.to_string();

                Ok(())
            })?;

            Err(error)
        }
        Err(error) => {
            rollback_files(&harness, &task_id, &error)?;

            Err(error)
        }
    }
}
