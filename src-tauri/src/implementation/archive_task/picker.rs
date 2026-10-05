use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::implementation::image_task::name::sort_sources;
use crate::result::{AppError, AppResult};

pub struct ImportSelection {
    pub source: PathBuf,
    pub images: Vec<PathBuf>,
}

/// # Errors
/// Only a native picker can introduce paths into an archive import task.
pub fn choose_import(app: &AppHandle, pair_images: bool) -> AppResult<Option<ImportSelection>> {
    let Some(source) = app
        .dialog()
        .file()
        .set_title("选择译文文件或项目 ZIP")
        .add_filter("译文与项目", &["json", "txt", "zip"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    let source = source.into_path().map_err(|_| AppError::InvalidInput)?;

    let mut images = Vec::new();

    if pair_images {
        let Some(folder) = app
            .dialog()
            .file()
            .set_title("选择完整配图目录（仅当前层）")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };

        let folder = folder.into_path().map_err(|_| AppError::InvalidInput)?;

        for entry in fs::read_dir(folder).map_err(|_| AppError::Storage)? {
            let entry = entry.map_err(|_| AppError::Storage)?;

            let kind = entry.file_type().map_err(|_| AppError::Storage)?;

            let path = entry.path();

            if kind.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        ["jpg", "jpeg", "png", "webp", "bmp"]
                            .iter()
                            .any(|allowed| extension.eq_ignore_ascii_case(allowed))
                    })
            {
                images.push(path);
            }
        }

        images = sort_sources(images)?
            .into_iter()
            .map(|(path, _)| path)
            .collect();
    }

    Ok(Some(ImportSelection { source, images }))
}

/// # Errors
/// The native save dialog authorizes the destination; replacing requires a separate confirmation.
pub fn choose_export(app: &AppHandle) -> AppResult<Option<(PathBuf, bool)>> {
    if !app.dialog().message("导出的 ZIP 同时包含完整 PRK 和兼容 LP。LP 仅保留最终文本与四位小数坐标，不保留双阶段、关注及校对确认；重新导入时优先使用 PRK。").title("导出项目").buttons(MessageDialogButtons::OkCancel).blocking_show() { return Ok(None); }

    let Some(target) = app
        .dialog()
        .file()
        .set_title("导出项目 ZIP")
        .set_file_name("translation.zip")
        .add_filter("项目 ZIP", &["zip"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let target = target.into_path().map_err(|_| AppError::InvalidInput)?;

    let exists = target.try_exists().map_err(|_| AppError::Storage)?;

    if exists
        && !app
            .dialog()
            .message("目标文件已经存在。完成导出后替换该文件？失败或取消会保留现有文件。")
            .title("确认覆盖")
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show()
    {
        return Ok(None);
    }

    Ok(Some((target, exists)))
}
