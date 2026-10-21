use std::fs::File;
use std::io::Read;
use std::path::Path;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::complex::terminology::import::{MAX_FILE_BYTES, parse};
use crate::data::terminology::TermbaseImport;
use crate::result::{AppError, AppResult};

/// # Errors
/// Bounds allocation even when a selected file grows while being read.
pub fn read(path: &Path) -> AppResult<TermbaseImport> {
    let file = File::open(path).map_err(|_| AppError::Storage)?;

    let metadata = file.metadata().map_err(|_| AppError::Storage)?;

    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(AppError::TermbaseImport);
    }

    let mut content = Vec::new();

    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut content)
        .map_err(|_| AppError::Storage)?;

    parse(&content)
}

/// # Errors
/// Only a native file picker introduces the import path; cancellation has no database effect.
pub fn choose(app: &AppHandle) -> AppResult<Option<TermbaseImport>> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("导入术语库 JSON")
        .add_filter("术语库 JSON", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    let path = file.into_path().map_err(|_| AppError::InvalidInput)?;

    read(&path).map(Some)
}
