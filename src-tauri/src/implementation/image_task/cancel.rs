use std::sync::Arc;

use crate::data::image_import::{ImageTaskPhase, ImageTaskStatus};
use crate::harness::Harness;
use crate::implementation::image_task::{prepare, recovery};
use crate::result::{AppError, AppResult};

/// # Errors
/// Commit verification must finish before resources can be cancelled or removed.
pub async fn cancel(harness: Arc<Harness>, task_id: String) -> AppResult<ImageTaskStatus> {
    let phase = harness.image_task.request_stop(&task_id)?;

    match phase {
        ImageTaskPhase::Ready => {
            let worker = Arc::clone(&harness);

            let worker_id = task_id.clone();

            tokio::task::spawn_blocking(move || {
                prepare::cleanup(
                    &worker,
                    &worker_id,
                    ImageTaskPhase::Cancelled,
                    "已取消图片准备".to_owned(),
                )
            })
            .await
            .map_err(|_| AppError::RecoveryRequired)??;

            harness.image_task.update(&task_id, |task| {
                task.permit = None;

                Ok(())
            })?;
        }
        ImageTaskPhase::Failed | ImageTaskPhase::Cancelled | ImageTaskPhase::Completed => {
            let _permit = Arc::clone(&harness.resource_task)
                .try_acquire_owned()
                .map_err(|_| AppError::Busy)?;

            recovery::retry(&harness, &task_id).await?;
        }
        _ => {}
    }

    harness.image_task.status(&task_id)
}
