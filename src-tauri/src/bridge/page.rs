use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::data::page::{RemovePages, ReorderPages};
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, execute};
use crate::model::page::Page;
use crate::usecase::page;

/// # Errors
/// Rejects stale page order, invalid identifiers, and unresolved writes.
#[tauri::command]
#[specta::specta]
pub async fn reorder_pages(
    state: State<'_, Arc<Harness>>,
    input: ReorderPages,
    request: WriteRequest,
) -> CommandResult<Vec<Page>> {
    let payload = serde_json::json!(["reorder_pages", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move { page::reorder_pages(harness.database().await?, input).await },
    )
    .await?)
}

/// # Errors
/// Rejects stale page identities and unresolved writes without deleting source files.
#[tauri::command]
#[specta::specta]
pub async fn remove_pages(
    state: State<'_, Arc<Harness>>,
    input: RemovePages,
    request: WriteRequest,
) -> CommandResult<Vec<Page>> {
    let payload = serde_json::json!(["remove_pages", input]);

    let harness = Arc::clone(state.inner());

    Ok(execute(
        Arc::clone(&harness.write),
        request,
        payload,
        move || async move {
            let pages = page::remove_pages(harness.database().await?, input).await?;

            // Cleanup never turns a confirmed content commit into a retryable mutation failure.
            if crate::implementation::image_task::recovery::recover(Arc::clone(&harness))
                .await
                .is_err()
            {
                eprintln!("Page deletion committed; resource cleanup remains pending");
            }

            Ok(pages)
        },
    )
    .await?)
}
