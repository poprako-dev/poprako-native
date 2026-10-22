use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::bridge::terminology::confirmation;
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermbaseAppend, TermbaseAppendResult, TerminologyContext};
use crate::harness::Harness;
use crate::implementation::terminology_import;
use crate::implementation::write::{WriteRequest, execute_with_confirmation};
use crate::result::AppError;
use crate::usecase::terminology::append;

/// # Errors
/// Reads a bounded native selection without exposing its path to the frontend.
#[tauri::command]
#[specta::specta]
pub async fn choose_termbase_append(app: AppHandle) -> CommandResult<Option<TermbaseAppend>> {
    Ok(
        tokio::task::spawn_blocking(move || terminology_import::choose_append(&app))
            .await
            .map_err(|_| AppError::Storage)??,
    )
}

/// # Errors
/// Deduplicates the original append request; uncertain retries only verify retained evidence.
#[tauri::command]
#[specta::specta]
pub async fn append_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    termbase_id: String,
    document: TermbaseAppend,
    request: WriteRequest,
) -> CommandResult<TermbaseAppendResult> {
    let payload = serde_json::json!(["append_termbase", access, termbase_id, document]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            append::append_termbase(
                harness.database().await?,
                access,
                termbase_id,
                document,
                evidence,
            )
            .await
        },
    )
    .await?)
}
