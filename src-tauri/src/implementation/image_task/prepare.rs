use std::path::PathBuf;
use std::sync::Arc;

use uuid::Uuid;

use crate::data::image_import::{
    ImageImportPreview, ImageImportTarget, ImageTaskPhase, ImageTaskStatus,
};
use crate::harness::Harness;
use crate::implementation::image::error::ImageError;
use crate::implementation::image_task::PreparedFile;
use crate::implementation::image_task::name::sort_sources;
use crate::result::{AppError, AppResult};
use crate::usecase::comic;

/// # Errors
/// Requires the complete preparation baseline to still describe the current target.
pub async fn validate_target(harness: &Harness, target: &ImageImportTarget) -> AppResult<()> {
    let detail = comic::get_comic_detail(harness.database().await?, target.comic_id()).await?;

    let matches = |page: &crate::model::page::Page, baseline: &crate::data::page::PageBaseline| {
        page.id == baseline.id
            && page.unit_revision == baseline.unit_revision
            && page.image.reference == baseline.image_reference
    };

    let valid = match target {
        ImageImportTarget::Append { baseline, .. } => {
            detail.pages.len() == baseline.len()
                && detail
                    .pages
                    .iter()
                    .zip(baseline)
                    .all(|(info, baseline)| matches(&info.page, baseline))
        }
        ImageImportTarget::Replace { baseline, .. } => detail
            .pages
            .iter()
            .any(|info| matches(&info.page, baseline)),
    };

    if !valid {
        return Err(AppError::Conflict);
    }

    Ok(())
}

/// # Errors
/// Retries retained cleanup before replacing a terminal task; never repeats its mutation.
pub async fn begin(
    harness: &Arc<Harness>,
    target: ImageImportTarget,
) -> AppResult<ImageTaskStatus> {
    validate_target(harness, &target).await?;

    let permit = Arc::clone(&harness.resource_task)
        .try_acquire_owned()
        .map_err(|_| AppError::Busy)?;

    if let Some(status) = harness.image_task.active_status()?
        && status.cleanup_pending
    {
        crate::implementation::image_task::recovery::retry(harness, &status.task_id).await?;
    }

    harness.image_task.begin(target, permit)
}

/// # Errors
/// Leaves failed cleanup files attached to the task for an explicit retry.
pub fn cleanup(
    harness: &Harness,
    task_id: &str,
    phase: ImageTaskPhase,
    message: String,
) -> AppResult<()> {
    let files = harness.image_task.update(task_id, |task| {
        task.status.phase = ImageTaskPhase::Stopping;

        Ok(std::mem::take(&mut task.files))
    })?;

    let mut pending = Vec::new();

    for file in files {
        if harness.resource.discard_prepared(&file.image).is_err() {
            pending.push(file);
        }
    }

    harness.image_task.update(task_id, |task| {
        task.status.phase = phase;

        task.status.message = message;

        task.status.cleanup_pending = !pending.is_empty();

        task.files = pending;

        task.status.previews.clear();

        Ok(())
    })
}

fn prepare_files(harness: &Harness, task_id: &str, paths: Vec<PathBuf>) -> AppResult<()> {
    let sources = sort_sources(paths)?;

    let target = harness
        .image_task
        .update(task_id, |task| Ok(task.target.clone()))?;

    if sources.is_empty()
        || matches!(target, ImageImportTarget::Replace { .. }) && sources.len() != 1
    {
        return Err(AppError::InvalidInput);
    }

    harness.image_task.update(task_id, |task| {
        if task.status.phase == ImageTaskPhase::Stopping {
            return Err(AppError::Busy);
        }

        task.status.phase = ImageTaskPhase::Preparing;

        task.status.total_files =
            u32::try_from(sources.len()).map_err(|_| AppError::InvalidInput)?;

        Ok(())
    })?;

    let task_uuid = Uuid::parse_str(task_id).map_err(|_| AppError::InvalidInput)?;

    for (source, original_name) in sources {
        let status = harness.image_task.status(task_id)?;

        if status.phase == ImageTaskPhase::Stopping {
            return Err(AppError::Busy);
        }

        let base_bytes = status.processed_bytes;

        let result =
            harness
                .resource
                .prepare_with_progress(&source, task_uuid, &harness.image, |copied| {
                    harness
                        .image_task
                        .update(task_id, |task| {
                            if task.status.phase == ImageTaskPhase::Stopping {
                                return Err(AppError::Busy);
                            }

                            task.status.processed_bytes = base_bytes
                                + f64::from(
                                    u32::try_from(copied).map_err(|_| AppError::InvalidInput)?,
                                );

                            Ok(())
                        })
                        .map_err(|_| ImageError::Cancelled)
                });

        let image = match result {
            Ok(image) => image,
            Err(error) => {
                harness.image_task.update(task_id, |task| {
                    task.status.message = error.to_string();

                    Ok(())
                })?;

                return Err(AppError::Storage);
            }
        };

        let handle = Uuid::new_v4().to_string();

        let byte_length =
            f64::from(u32::try_from(image.byte_length).map_err(|_| AppError::InvalidInput)?);

        let preview = ImageImportPreview {
            handle: handle.clone(),
            original_name: original_name.clone(),
            width: image.width,
            height: image.height,
            byte_length,
        };

        harness.image_task.update(task_id, |task| {
            task.files.push(PreparedFile {
                image,
                original_name,
                handle,
            });

            task.status.previews.push(preview);

            task.status.processed_files += 1;

            Ok(())
        })?;
    }

    harness.image_task.update(task_id, |task| {
        if task.status.phase == ImageTaskPhase::Stopping {
            return Err(AppError::Busy);
        }

        task.status.phase = ImageTaskPhase::Ready;

        Ok(())
    })
}

/// # Errors
/// Performs preparation off the async runtime, leaving task completion to the caller.
pub async fn run(harness: Arc<Harness>, task_id: String, paths: Vec<PathBuf>) -> AppResult<()> {
    tokio::task::spawn_blocking(move || prepare_files(&harness, &task_id, paths))
        .await
        .map_err(|_| AppError::RecoveryRequired)?
}
