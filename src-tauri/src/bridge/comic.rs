use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::data::comic::{ChapterOrder, ComicInfo, ComicMetadata, MoveChapter, NewComic};
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, execute};
use crate::model::chapter::Chapter;
use crate::model::comic::Comic;
use crate::usecase::comic;

/// # Errors
/// Rejects invalid pagination or unavailable storage.
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
/// Preserves exact write outcomes and rejects invalid or stale requests.
#[tauri::command]
#[specta::specta]
pub async fn create_comic(
    state: State<'_, Arc<Harness>>,
    input: NewComic,
    request: WriteRequest,
) -> CommandResult<Chapter> {
    let payload = serde_json::json!(["create_comic", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { comic::create_comic(harness.database().await?, input).await },
    )
    .await?)
}

/// # Errors
/// Preserves exact write outcomes and rejects invalid or stale requests.
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
/// Preserves exact write outcomes and rejects invalid or stale requests.
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
/// Preserves exact write outcomes and rejects invalid or stale requests.
#[tauri::command]
#[specta::specta]
pub async fn reorder_chapters(
    state: State<'_, Arc<Harness>>,
    input: ChapterOrder,
    request: WriteRequest,
) -> CommandResult<()> {
    let payload = serde_json::json!(["reorder_chapters", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { comic::reorder_chapters(harness.database().await?, input).await },
    )
    .await?)
}

/// # Errors
/// Preserves exact write outcomes and rejects invalid or stale requests.
#[tauri::command]
#[specta::specta]
pub async fn move_chapter(
    state: State<'_, Arc<Harness>>,
    input: MoveChapter,
    request: WriteRequest,
) -> CommandResult<Chapter> {
    let payload = serde_json::json!(["move_chapter", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { comic::move_chapter(harness.database().await?, input).await },
    )
    .await?)
}
