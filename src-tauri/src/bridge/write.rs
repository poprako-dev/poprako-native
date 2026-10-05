use std::sync::Arc;

use tauri::State;

use crate::bridge::error::CommandResult;
use crate::harness::Harness;
use crate::implementation::write::{WriteRequest, WriteSession, WriteStatus};

/// # Errors
/// Reports unavailable write-ledger state.
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri deserializes command arguments into owned invocation values"
)]
#[tauri::command]
#[specta::specta]
pub fn get_write_session(state: State<'_, Arc<Harness>>) -> CommandResult<WriteSession> {
    Ok(state.write.session()?)
}

/// # Errors
/// Rejects invalid session ownership or unavailable verification state.
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri deserializes command arguments into owned invocation values"
)]
#[tauri::command]
#[specta::specta]
pub fn get_write_result(
    state: State<'_, Arc<Harness>>,
    request: WriteRequest,
) -> CommandResult<WriteStatus> {
    state.write.refresh(&request)?;

    Ok(state.write.status(&request)?)
}

/// # Errors
/// Rejects pending or uncertain results instead of discarding their evidence.
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri deserializes command arguments into owned invocation values"
)]
#[tauri::command]
#[specta::specta]
pub fn acknowledge_write_result(
    state: State<'_, Arc<Harness>>,
    request: WriteRequest,
) -> CommandResult<()> {
    Ok(state.write.acknowledge(&request)?)
}
