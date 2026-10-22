use std::fs::File;
use std::io::Read;
use std::path::Path;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::complex::terminology::append;
use crate::complex::terminology::import::{MAX_FILE_BYTES, parse};
use crate::data::terminology::{TermbaseAppend, TermbaseImport};
use crate::result::{AppError, AppResult};

/// # Errors
/// Bounds allocation even when a selected file grows while being read.
fn read_content(path: &Path) -> AppResult<Vec<u8>> {
    let file = File::open(path).map_err(|_| AppError::Storage)?;

    let metadata = file.metadata().map_err(|_| AppError::Storage)?;

    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(AppError::TermbaseImport);
    }

    let mut content = Vec::new();

    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut content)
        .map_err(|_| AppError::Storage)?;

    Ok(content)
}

fn choose_content(app: &AppHandle, title: &str) -> AppResult<Option<Vec<u8>>> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter("术语库 JSON", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    let path = file.into_path().map_err(|_| AppError::InvalidInput)?;

    read_content(&path).map(Some)
}

/// # Errors
/// Only a native file picker introduces the import path; cancellation has no database effect.
pub fn choose(app: &AppHandle) -> AppResult<Option<TermbaseImport>> {
    choose_content(app, "导入术语库 JSON")?
        .map(|content| parse(&content))
        .transpose()
}

/// # Errors
/// Shares bounded file reading while accepting only the metadata-free append document.
pub fn choose_append(app: &AppHandle) -> AppResult<Option<TermbaseAppend>> {
    choose_content(app, "追加术语 JSON")?
        .map(|content| append::parse(&content))
        .transpose()
}
