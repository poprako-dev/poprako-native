use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::data::comic::{ComicDetail, ComicInfo, ComicMetadata};
use crate::data::editor::PageEditor;
use crate::data::search::{ReplaceUnits, ReplacementResult, SearchHit, SearchUnits};
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, execute};
use crate::model::comic::Comic;
use crate::model::preference::ApplicationPreference;
use crate::model::work_position::WorkPosition;
use crate::usecase::{comic, editor, preference, work_position};

/// # Errors
/// Rejects invalid pagination or unavailable local storage.
#[tauri::command]
#[specta::specta]
pub async fn list_comic_infos(
    state: State<'_, Arc<Harness>>,
    offset: u32,
    limit: u32,
) -> CommandResult<Vec<ComicInfo>> {
    Ok(comic::list_comic_infos(state.database().await?, offset, limit).await?)
}

/// # Errors
/// Reports invalid identifiers, missing projects, or unreadable storage.
#[tauri::command]
#[specta::specta]
pub async fn get_comic_detail(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
) -> CommandResult<ComicDetail> {
    Ok(comic::get_comic_detail(state.database().await?, &comic_id).await?)
}

/// # Errors
/// Rejects invalid metadata or conflicting write requests; preserves uncertain results.
#[tauri::command]
#[specta::specta]
pub async fn create_comic(
    state: State<'_, Arc<Harness>>,
    metadata: ComicMetadata,
    request: WriteRequest,
) -> CommandResult<Comic> {
    let payload = serde_json::json!(["create_comic", metadata]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { comic::create_comic(harness.database().await?, metadata).await },
    )
    .await?)
}

/// # Errors
/// Reports invalid metadata, missing projects, or unresolved writes.
#[tauri::command]
#[specta::specta]
pub async fn update_comic_metadata(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    metadata: ComicMetadata,
    request: WriteRequest,
) -> CommandResult<Comic> {
    let payload = serde_json::json!(["update_comic_metadata", comic_id, metadata]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move {
            comic::update_comic_metadata(harness.database().await?, &comic_id, metadata).await
        },
    )
    .await?)
}

/// # Errors
/// Reports missing projects, conflicting requests, or unresolved deletion.
#[tauri::command]
#[specta::specta]
pub async fn delete_comic(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    request: WriteRequest,
) -> CommandResult<()> {
    let payload = serde_json::json!(["delete_comic", comic_id]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { comic::delete_comic(harness.database().await?, &comic_id).await },
    )
    .await?)
}

/// # Errors
/// Reports invalid page ownership, missing content, or unreadable storage.
#[tauri::command]
#[specta::specta]
pub async fn get_page_editor(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    page_id: String,
) -> CommandResult<PageEditor> {
    Ok(editor::get_page_editor(state.database().await?, &comic_id, &page_id).await?)
}

/// # Errors
/// Reports migration or local storage failures.
#[tauri::command]
#[specta::specta]
pub async fn get_preference(
    state: State<'_, Arc<Harness>>,
) -> CommandResult<ApplicationPreference> {
    Ok(preference::get_preference(state.database().await?).await?)
}

#[must_use]
#[tauri::command]
#[specta::specta]
pub fn get_default_preference() -> ApplicationPreference {
    ApplicationPreference::default()
}

/// # Errors
/// Rejects invalid preferences, stale baselines, or unresolved writes.
#[tauri::command]
#[specta::specta]
pub async fn update_preference(
    state: State<'_, Arc<Harness>>,
    baseline: ApplicationPreference,
    value: ApplicationPreference,
    request: WriteRequest,
) -> CommandResult<ApplicationPreference> {
    let payload = serde_json::json!(["update_preference", baseline, value]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move {
            preference::update_preference(harness.database().await?, baseline, value).await
        },
    )
    .await?)
}

/// # Errors
/// Rejects invalid project/page/unit ownership or unresolved writes.
#[tauri::command]
#[specta::specta]
pub async fn update_work_position(
    state: State<'_, Arc<Harness>>,
    position: WorkPosition,
    request: WriteRequest,
) -> CommandResult<WorkPosition> {
    let payload = serde_json::json!(["update_work_position", position]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move {
            work_position::update_work_position(harness.database().await?, position).await
        },
    )
    .await?)
}

/// # Errors
/// Rejects empty queries, more than 100 hits, or unreadable content.
#[tauri::command]
#[specta::specta]
pub async fn search_units(
    state: State<'_, Arc<Harness>>,
    input: SearchUnits,
) -> CommandResult<Vec<SearchHit>> {
    Ok(crate::usecase::search::search_units(state.database().await?, input).await?)
}

/// # Errors
/// Rejects stale search hits, invalid rules, or unresolved writes.
#[tauri::command]
#[specta::specta]
pub async fn replace_units(
    state: State<'_, Arc<Harness>>,
    input: ReplaceUnits,
    request: WriteRequest,
) -> CommandResult<ReplacementResult> {
    let payload = serde_json::json!(["replace_units", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move {
            crate::usecase::search::replace_units(harness.database().await?, input).await
        },
    )
    .await?)
}
