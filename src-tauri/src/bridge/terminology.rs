use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::bridge::error::CommandResult;
use crate::data::recovery::EvidenceHandle;
use crate::data::terminology::{TermbaseInfo, TerminologyContext, TerminologyQuery};
use crate::harness::Harness;
use crate::implementation::write::Confirmation;
use crate::model::terminology::Term;
use crate::result::AppError;
use crate::usecase::terminology;

pub mod append;
pub mod export;
pub mod import;
pub mod term;
pub mod termbase;

#[must_use]
pub fn confirmation<T: Clone + Send + Sync + Serialize + 'static>(
    harness: Arc<Harness>,
    evidence: EvidenceHandle<T>,
) -> Confirmation {
    Arc::new(move || {
        let harness = Arc::clone(&harness);

        let evidence = Arc::clone(&evidence);

        Box::pin(async move {
            let retained = evidence
                .lock()
                .map_err(|_| AppError::RecoveryRequired)?
                .clone()
                .ok_or(AppError::CommitUncertain)?;

            let value =
                crate::usecase::recovery::verify_evidence(harness.database().await?, &retained)
                    .await?;

            serde_json::to_value(value).map_err(|_| AppError::RecoveryRequired)
        })
    })
}

/// # Errors
/// Lists libraries visible in the given context.
#[tauri::command]
#[specta::specta]
pub async fn list_termbase_infos(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    query: TerminologyQuery,
) -> CommandResult<Vec<TermbaseInfo>> {
    Ok(terminology::list_termbase_infos(state.database().await?, access, query).await?)
}

/// # Errors
/// Rejects missing libraries or mismatched ownership.
#[tauri::command]
#[specta::specta]
pub async fn get_termbase(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    id: String,
) -> CommandResult<TermbaseInfo> {
    Ok(terminology::get_termbase(state.database().await?, access, id).await?)
}

/// # Errors
/// Lists a bounded page of matching terms from one visible library.
#[tauri::command]
#[specta::specta]
pub async fn list_term_infos(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    termbase_id: String,
    query: TerminologyQuery,
) -> CommandResult<Vec<Term>> {
    Ok(terminology::list_term_infos(state.database().await?, access, termbase_id, query).await?)
}

/// # Errors
/// Copies a validated library's preferred translation without modifying editor state.
#[tauri::command]
#[specta::specta]
pub async fn copy_term_target(
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    id: String,
) -> CommandResult<()> {
    let text = terminology::preferred_translation(state.database().await?, access, id).await?;

    Ok(crate::implementation::clipboard::write(text)?)
}
