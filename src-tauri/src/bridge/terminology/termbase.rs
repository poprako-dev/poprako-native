use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::bridge::terminology::confirmation;
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermbaseMetadata, TerminologyContext};
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, execute_with_confirmation};
use crate::model::terminology::Termbase;
use crate::usecase::terminology;

/// # Errors
/// Validates scope and retains exact transaction evidence for retrying confirmation.
#[tauri::command]
#[specta::specta]
pub async fn create_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    metadata: TermbaseMetadata,
    request: WriteRequest,
) -> CommandResult<Termbase> {
    let payload = serde_json::json!(["create_termbase", access, metadata]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::create_termbase(harness.database().await?, access, metadata, evidence)
                .await
        },
    )
    .await?)
}

/// # Errors
/// Validates scope and retains exact transaction evidence for retrying confirmation.
#[tauri::command]
#[specta::specta]
pub async fn update_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    baseline: Termbase,
    metadata: TermbaseMetadata,
    request: WriteRequest,
) -> CommandResult<Termbase> {
    let payload = serde_json::json!(["update_termbase", access, baseline, metadata]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::update_termbase(
                harness.database().await?,
                access,
                baseline,
                metadata,
                evidence,
            )
            .await
        },
    )
    .await?)
}

/// # Errors
/// Validates scope and retains exact transaction evidence for retrying confirmation.
#[tauri::command]
#[specta::specta]
pub async fn delete_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    baseline: Termbase,
    request: WriteRequest,
) -> CommandResult<()> {
    let payload = serde_json::json!(["delete_termbase", access, baseline]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::delete_termbase(harness.database().await?, access, baseline, evidence)
                .await
        },
    )
    .await?)
}
