use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::bridge::terminology::confirmation;
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermbaseImport, TerminologyContext};
use crate::harness::Harness;
use crate::implementation::terminology_import;
use crate::implementation::write::{WriteRequest, execute_with_confirmation};
use crate::model::terminology::Termbase;
use crate::result::AppError;
use crate::usecase::terminology::import;

/// # Errors
/// Reads and validates a native selection without changing the database or exposing paths.
#[tauri::command]
#[specta::specta]
pub async fn choose_termbase_import(app: AppHandle) -> CommandResult<Option<TermbaseImport>> {
    Ok(
        tokio::task::spawn_blocking(move || terminology_import::choose(&app))
            .await
            .map_err(|_| AppError::Storage)??,
    )
}

/// # Errors
/// Imports the fixed selected document once; uncertain results only verify retained evidence.
#[tauri::command]
#[specta::specta]
pub async fn import_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    document: TermbaseImport,
    request: WriteRequest,
) -> CommandResult<Termbase> {
    let payload = serde_json::json!(["import_termbase", access, document]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            import::import_termbase(harness.database().await?, access, document, evidence).await
        },
    )
    .await?)
}
