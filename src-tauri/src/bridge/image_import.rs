use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::data::image_import::{ImageImportTarget, ImageTaskStatus};
use crate::harness::Harness;
use crate::implementation::image_selection::NativeImageSelection;
use crate::implementation::write::WriteRequest;
use crate::model::page::Page;
use crate::part::image_selection::ImageSelectionKind;
use crate::usecase::image_import;

/// # Errors
/// Returns validation, task state, or storage errors without discarding pending resources.
#[tauri::command]
#[specta::specta]
pub async fn select_images(
    app: AppHandle,
    state: State<'_, Arc<Harness>>,
    target: ImageImportTarget,
) -> CommandResult<Option<ImageTaskStatus>> {
    Ok(Some(
        image_import::select(
            Arc::clone(state.inner()),
            NativeImageSelection::new(app),
            target,
            ImageSelectionKind::Files,
        )
        .await?,
    ))
}

/// # Errors
/// Returns validation, task state, or storage errors without discarding pending resources.
#[tauri::command]
#[specta::specta]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands require owned arguments and managed state extraction."
)]
pub fn get_image_import(
    state: State<'_, Arc<Harness>>,
    task_id: String,
) -> CommandResult<ImageTaskStatus> {
    Ok(image_import::get_status(&state, &task_id)?)
}

/// # Errors
/// Returns validation, task state, or storage errors without discarding pending resources.
#[tauri::command]
#[specta::specta]
pub async fn cancel_image_import(
    state: State<'_, Arc<Harness>>,
    task_id: String,
) -> CommandResult<ImageTaskStatus> {
    Ok(image_import::cancel(Arc::clone(state.inner()), task_id).await?)
}

/// # Errors
/// Returns validation, task state, or storage errors without discarding pending resources.
#[tauri::command]
#[specta::specta]
pub async fn confirm_image_import(
    state: State<'_, Arc<Harness>>,
    task_id: String,
    clear_units: bool,
    request: WriteRequest,
) -> CommandResult<Vec<Page>> {
    Ok(image_import::confirm(Arc::clone(state.inner()), task_id, clear_units, request).await?)
}

/// # Errors
/// Rejects unavailable targets or an already active resource task before opening a dialog.
#[tauri::command]
#[specta::specta]
pub async fn select_image_folder(
    app: AppHandle,
    state: State<'_, Arc<Harness>>,
    target: ImageImportTarget,
) -> CommandResult<Option<ImageTaskStatus>> {
    Ok(Some(
        image_import::select(
            Arc::clone(state.inner()),
            NativeImageSelection::new(app),
            target,
            ImageSelectionKind::Folder,
        )
        .await?,
    ))
}
