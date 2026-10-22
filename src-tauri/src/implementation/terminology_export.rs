use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tempfile::NamedTempFile;

use crate::data::terminology::TermbaseExportOutcome;
use crate::data::terminology::portable::PortableTermbase;
use crate::result::{AppError, AppResult};

#[must_use]
pub fn filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .take(60)
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            value if value.is_control() => '_',
            value => value,
        })
        .collect();

    let cleaned = cleaned.trim_matches([' ', '.']);

    if cleaned.is_empty() {
        return "termbase.json".to_owned();
    }

    let stem = cleaned
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();

    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));

    match reserved {
        true => format!("_{cleaned}.json"),
        false => format!("{cleaned}.json"),
    }
}

/// # Errors
/// Only the native save picker may authorize a destination and its replacement.
fn choose(app: &AppHandle, name: &str) -> AppResult<Option<(PathBuf, bool)>> {
    let window = app.get_webview_window("main").ok_or(AppError::NotFound)?;

    let Some(file) = app
        .dialog()
        .file()
        .set_title("导出术语库 JSON")
        .set_file_name(filename(name))
        .add_filter("术语库 JSON", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let target = file.into_path().map_err(|_| AppError::InvalidInput)?;

    let exists = target.try_exists().map_err(|_| AppError::Storage)?;

    // Keep the confirmation visible after the native save panel closes on macOS.
    if exists
        && !app
            .dialog()
            .message("目标文件已经存在。完成导出后替换该文件？失败或取消会保留现有文件。")
            .parent(&window)
            .title("确认覆盖")
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show()
    {
        return Ok(None);
    }

    Ok(Some((target, exists)))
}

/// # Errors
/// Publishes only a complete synchronized JSON file; failed writes preserve the destination.
pub fn write(target: &Path, document: &PortableTermbase, overwrite: bool) -> AppResult<()> {
    let parent = target.parent().ok_or(AppError::InvalidInput)?;

    let mut temporary = NamedTempFile::new_in(parent).map_err(|_| AppError::Storage)?;

    let mut writer = BufWriter::new(temporary.as_file_mut());

    serde_json::to_writer(&mut writer, document).map_err(|_| AppError::Storage)?;

    writer.flush().map_err(|_| AppError::Storage)?;

    drop(writer);

    temporary
        .as_file()
        .sync_all()
        .map_err(|_| AppError::Storage)?;

    match overwrite {
        true => temporary.persist(target).map_err(|_| AppError::Storage)?,
        false => temporary
            .persist_noclobber(target)
            .map_err(|_| AppError::Storage)?,
    };

    Ok(())
}

/// # Errors
/// Retains the selected snapshot through the native dialog without holding a database transaction.
pub fn save(app: &AppHandle, document: &PortableTermbase) -> AppResult<TermbaseExportOutcome> {
    let Some((target, overwrite)) = choose(app, &document.name)? else {
        return Ok(TermbaseExportOutcome::Cancelled);
    };

    write(&target, document, overwrite)?;

    Ok(TermbaseExportOutcome::Saved)
}

#[cfg(test)]
mod test;
