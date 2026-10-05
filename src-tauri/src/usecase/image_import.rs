use std::path::PathBuf;
use std::sync::Arc;

use poprako_orchestra::Run;

use crate::data::image_import::{ImageImportTarget, ImageTaskPhase, ImageTaskStatus};
use crate::harness::Harness;
use crate::implementation::image_task::{cancel, commit, prepare};
use crate::implementation::write::{WriteRequest, execute};
use crate::model::page::Page;
use crate::part::image_selection::{ImageSelectionKind, SelectImages};
use crate::result::{AppError, AppResult};

async fn finish(harness: Arc<Harness>, task_id: String, phase: ImageTaskPhase, message: String) {
    let result = tokio::task::spawn_blocking(move || {
        prepare::cleanup(&harness, &task_id, phase, message)?;

        harness.image_task.update(&task_id, |task| {
            task.permit = None;

            Ok(())
        })
    })
    .await
    .map_err(|_| AppError::RecoveryRequired)
    .and_then(|result| result);

    if result.is_err() {
        eprintln!("Unable to finish image task cleanup; startup recovery is required");
    }
}

async fn run_preparation(harness: Arc<Harness>, task_id: String, paths: Vec<PathBuf>) {
    let Err(error) = prepare::run(Arc::clone(&harness), task_id.clone(), paths).await else {
        return;
    };

    let status = harness.image_task.status(&task_id);

    let (phase, message) = match status {
        Ok(status) if status.phase == ImageTaskPhase::Stopping => {
            (ImageTaskPhase::Cancelled, "已取消图片准备".to_owned())
        }
        Ok(status) if !status.message.is_empty() => (ImageTaskPhase::Failed, status.message),
        _ => (ImageTaskPhase::Failed, error.to_string()),
    };

    finish(harness, task_id, phase, message).await;
}

#[cfg(test)]
pub fn start_preparation(harness: Arc<Harness>, task_id: String, paths: Vec<PathBuf>) {
    tokio::spawn(run_preparation(harness, task_id, paths));
}

async fn run_selection<P>(
    harness: Arc<Harness>,
    task_id: String,
    selection: P,
    kind: ImageSelectionKind,
) where
    P: Run<SelectImages, Error = AppError>,
{
    let selected = selection.run(&SelectImages { kind }).await;

    let Ok(status) = harness.image_task.status(&task_id) else {
        return;
    };

    if status.phase == ImageTaskPhase::Stopping {
        finish(
            harness,
            task_id,
            ImageTaskPhase::Cancelled,
            "已取消选择".to_owned(),
        )
        .await;

        return;
    }

    let (phase, message) = match selected {
        Ok(Some(paths)) if !paths.is_empty() => {
            run_preparation(harness, task_id, paths).await;

            return;
        }
        Ok(Some(_)) => (
            ImageTaskPhase::Failed,
            "所选文件夹中没有支持的图片，请选择包含静态图片的文件夹".to_owned(),
        ),
        Ok(None) => (ImageTaskPhase::Cancelled, "已取消选择".to_owned()),
        Err(error) => (ImageTaskPhase::Failed, error.to_string()),
    };

    finish(harness, task_id, phase, message).await;
}

/// # Errors
/// Rejects stale targets and concurrent resource work before starting the native selection.
pub async fn select<P>(
    harness: Arc<Harness>,
    selection: P,
    target: ImageImportTarget,
    kind: ImageSelectionKind,
) -> AppResult<ImageTaskStatus>
where
    P: Run<SelectImages, Error = AppError> + Send + 'static,
{
    harness.ensure_resources().await?;

    let status = prepare::begin(&harness, target).await?;

    // The dialog and preparation outlive cancellation of the initiating IPC future.
    tokio::spawn(run_selection(
        harness,
        status.task_id.clone(),
        selection,
        kind,
    ));

    Ok(status)
}

/// # Errors
/// A task from an earlier process or already replaced task cannot be inspected.
pub fn get_status(harness: &Harness, task_id: &str) -> AppResult<ImageTaskStatus> {
    harness.image_task.status(task_id)
}

/// # Errors
/// Commit verification must finish before resources can be cancelled or removed.
pub async fn cancel(harness: Arc<Harness>, task_id: String) -> AppResult<ImageTaskStatus> {
    cancel::cancel(harness, task_id).await
}

/// # Errors
/// Stale previews and failed commits retain their explicit task outcome.
pub async fn confirm(
    harness: Arc<Harness>,
    task_id: String,
    clear_units: bool,
    request: WriteRequest,
) -> AppResult<Vec<Page>> {
    let payload = serde_json::json!(["confirm_image_import", task_id, clear_units]);

    execute(Arc::clone(&harness.write), request, payload, move || {
        commit::confirm(harness, task_id, clear_units)
    })
    .await
}

#[cfg(test)]
mod test;
