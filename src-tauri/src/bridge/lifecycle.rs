use std::sync::Arc;

use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::bridge::error::CommandResult;
use crate::harness::Harness;

/// # Errors
/// Refuses closing while accepted writes are pending or uncertain.
#[tauri::command]
#[specta::specta]
pub async fn request_exit(app: AppHandle, state: State<'_, Arc<Harness>>) -> CommandResult<()> {
    let database = state.database().await;

    let _resources = Arc::clone(&state.resource_task)
        .try_acquire_owned()
        .map_err(|_| crate::result::AppError::Busy)?;

    state.write.begin_close()?;

    if let Ok(database) = database {
        database.pool().close().await;
    }

    state.allow_exit.store(true, Ordering::Release);

    app.exit(0);

    Ok(())
}
