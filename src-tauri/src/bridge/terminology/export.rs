use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::data::terminology::{TermbaseExportOutcome, TerminologyContext};
use crate::harness::Harness;
use crate::implementation::terminology_export;
use crate::result::AppError;
use crate::usecase::terminology::export;

/// # Errors
/// Saves a visible library through a native picker without exposing paths or changing the database.
#[tauri::command]
#[specta::specta]
pub async fn export_termbase(
    app: AppHandle,
    state: State<'_, Arc<Harness>>,
    access: TerminologyContext,
    id: String,
) -> CommandResult<TermbaseExportOutcome> {
    let document = export::export_termbase(state.database().await?, access, id).await?;

    Ok(
        tokio::task::spawn_blocking(move || terminology_export::save(&app, &document))
            .await
            .map_err(|_| AppError::Storage)??,
    )
}
