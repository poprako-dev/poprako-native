use std::sync::{Arc, Mutex};

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::data::editor::{PageEditor, SavePageUnits};
use crate::harness::Harness;
use crate::implementation::write::{Confirmation, WriteRequest, execute_with_confirmation};
use crate::result::AppError;
use crate::usecase::editor::{save_page_units_with_evidence, verify_page_save};

/// # Errors
/// Rejects stale snapshots and retains exact evidence for uncertain commits.
#[tauri::command]
#[specta::specta]
pub async fn save_page_units(
    state: State<'_, Arc<Harness>>,
    snapshot: SavePageUnits,
    request: WriteRequest,
) -> CommandResult<PageEditor> {
    let payload = serde_json::json!(["save_page_units", snapshot]);

    let harness = Arc::clone(state.inner());

    let evidence = Arc::new(Mutex::new(None));

    let confirmation_harness = Arc::clone(&harness);

    let confirmation_evidence = Arc::clone(&evidence);

    let confirmation: Confirmation = Arc::new(move || {
        let harness = Arc::clone(&confirmation_harness);

        let evidence = Arc::clone(&confirmation_evidence);

        Box::pin(async move {
            let retained = evidence
                .lock()
                .map_err(|_| AppError::RecoveryRequired)?
                .clone()
                .ok_or(AppError::CommitUncertain)?;

            let result = verify_page_save(harness.database().await?, &retained).await?;

            serde_json::to_value(result).map_err(|_| AppError::RecoveryRequired)
        })
    });

    Ok(execute_with_confirmation(
        Arc::clone(&harness.write),
        request,
        payload,
        Some(confirmation),
        move || async move {
            save_page_units_with_evidence(harness.database().await?, snapshot, evidence).await
        },
    )
    .await?)
}
