use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::data::archive_task::{
    ArchiveCompletion, ArchiveImportMode, ArchiveSelection, ArchiveTarget, ArchiveTask,
};
use crate::data::comic::ComicMetadata;
use crate::data::search::TextStage;
use crate::harness::Harness;
use crate::implementation::archive_task::{commit, export, picker, prepare};
use crate::implementation::write::{WriteRequest, execute};
use crate::result::AppError;

/// # Errors
/// Rejects concurrent resource work or an invalid native selection.
#[tauri::command]
#[specta::specta]
pub async fn choose_archive_import(
    app: AppHandle,
    state: State<'_, Arc<Harness>>,
    target: ArchiveTarget,
    text_stage: TextStage,
    pair_external_images: bool,
) -> CommandResult<ArchiveSelection> {
    let state = Arc::clone(&state);

    state.ensure_resources().await?;

    let permit = Arc::clone(&state.resource_task)
        .try_acquire_owned()
        .map_err(|_| AppError::Busy)?;

    let selection =
        tokio::task::spawn_blocking(move || picker::choose_import(&app, pair_external_images))
            .await
            .map_err(|_| AppError::Storage)??;

    let Some(selection) = selection else {
        return Ok(ArchiveSelection::Cancelled);
    };

    let (id, cancel) = state.archive_tasks.create(permit)?;

    tokio::spawn(prepare::run(
        state,
        id.clone(),
        selection,
        target,
        text_stage,
        cancel,
    ));

    Ok(ArchiveSelection::Selected { task_id: id })
}

/// # Errors
/// Unknown or consumed handles are not valid task identities.
#[tauri::command]
#[specta::specta]
pub async fn get_archive_task(
    state: State<'_, Arc<Harness>>,
    task_id: String,
) -> CommandResult<ArchiveTask> {
    Ok(state.archive_tasks.get(&task_id)?)
}

/// # Errors
/// Committing or uncertain work cannot be abandoned.
#[tauri::command]
#[specta::specta]
pub async fn cancel_archive_task(
    state: State<'_, Arc<Harness>>,
    task_id: String,
) -> CommandResult<()> {
    Ok(prepare::cancel(Arc::clone(&state), task_id).await?)
}

/// # Errors
/// Pending and uncertain outcomes cannot be discarded.
#[tauri::command]
#[specta::specta]
pub async fn acknowledge_archive_task(
    state: State<'_, Arc<Harness>>,
    task_id: String,
) -> CommandResult<()> {
    Ok(state.archive_tasks.acknowledge(&task_id)?)
}

/// # Errors
/// Stale previews, invalid input and failed transactions retain explicit outcomes.
#[tauri::command]
#[specta::specta]
pub async fn confirm_archive_import(
    state: State<'_, Arc<Harness>>,
    task_id: String,
    metadata: ComicMetadata,
    mode: ArchiveImportMode,
    request: WriteRequest,
) -> CommandResult<ArchiveCompletion> {
    let state = Arc::clone(&state);

    let payload = serde_json::json!(["confirm_archive_import", task_id, metadata, mode]);

    Ok(
        execute(Arc::clone(&state.write), request, payload, move || {
            commit::run(state, task_id, metadata, mode)
        })
        .await?,
    )
}

/// # Errors
/// Only a native save dialog may authorize the export destination.
#[tauri::command]
#[specta::specta]
pub async fn export_archive(
    app: AppHandle,
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    include_images: bool,
) -> CommandResult<ArchiveSelection> {
    let state = Arc::clone(&state);

    state.ensure_resources().await?;

    let permit = Arc::clone(&state.resource_task)
        .try_acquire_owned()
        .map_err(|_| AppError::Busy)?;

    let destination = tokio::task::spawn_blocking(move || picker::choose_export(&app))
        .await
        .map_err(|_| AppError::Storage)??;

    let Some((target, overwrite)) = destination else {
        return Ok(ArchiveSelection::Cancelled);
    };

    let (id, cancel) = state.archive_tasks.create(permit)?;

    tokio::spawn(export::run(
        state,
        id.clone(),
        comic_id,
        target,
        include_images,
        overwrite,
        cancel,
    ));

    Ok(ArchiveSelection::Selected { task_id: id })
}
