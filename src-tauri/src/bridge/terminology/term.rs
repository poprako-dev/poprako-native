use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::bridge::terminology::confirmation;
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermMetadata, TerminologyContext};
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, execute_with_confirmation};
use crate::model::terminology::Term;
use crate::usecase::terminology;

/// # Errors
/// Validates scope and retains exact transaction evidence for retrying confirmation.
#[tauri::command]
#[specta::specta]
pub async fn create_term(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    termbase_id: String,
    metadata: TermMetadata,
    request: WriteRequest,
) -> CommandResult<Term> {
    let payload = serde_json::json!(["create_term", access, termbase_id, metadata]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::create_term(
                harness.database().await?,
                access,
                termbase_id,
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
pub async fn update_term(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    baseline: Term,
    metadata: TermMetadata,
    request: WriteRequest,
) -> CommandResult<Term> {
    let payload = serde_json::json!(["update_term", access, baseline, metadata]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::update_term(
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
pub async fn delete_term(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    baseline: Term,
    request: WriteRequest,
) -> CommandResult<()> {
    let payload = serde_json::json!(["delete_term", access, baseline]);

    let harness = Arc::clone(state.inner());

    let evidence = evidence_handle();

    let confirm = confirmation(Arc::clone(&harness), Arc::clone(&evidence));

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirm),
        move || async move {
            terminology::delete_term(harness.database().await?, access, baseline, evidence).await
        },
    )
    .await?)
}
