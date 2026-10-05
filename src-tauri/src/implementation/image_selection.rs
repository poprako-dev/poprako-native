use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};
use poprako_orchestra::Run;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::part::image_selection::{ImageSelectionKind, SelectImages};
use crate::result::{AppError, AppResult};

fn folder_sources(folder: &Path) -> AppResult<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in std::fs::read_dir(folder).map_err(|_| AppError::Storage)? {
        let entry = entry.map_err(|_| AppError::Storage)?;

        if !entry.file_type().map_err(|_| AppError::Storage)?.is_file() {
            continue;
        }

        let path = entry.path();

        let expected_image = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "bmp"
                )
            });

        let format = ImageReader::open(&path)
            .and_then(ImageReader::with_guessed_format)
            .map_err(|_| AppError::Storage)?
            .format();

        if expected_image
            || matches!(
                format,
                Some(ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP | ImageFormat::Bmp)
            )
        {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn choose(app: &AppHandle, kind: ImageSelectionKind) -> AppResult<Option<Vec<PathBuf>>> {
    let dialog = app.dialog().file();

    if matches!(kind, ImageSelectionKind::Folder) {
        let Some(folder) = dialog.set_title("选择图片文件夹").blocking_pick_folder() else {
            return Ok(None);
        };

        let path = folder.into_path().map_err(|_| AppError::InvalidInput)?;

        return Ok(Some(folder_sources(&path)?));
    }

    dialog
        .set_title("选择本地图片")
        .add_filter("图片", &["jpg", "jpeg", "png", "webp", "bmp"])
        .blocking_pick_files()
        .map(|selection| {
            selection
                .into_iter()
                .map(|file| file.into_path().map_err(|_| AppError::InvalidInput))
                .collect()
        })
        .transpose()
}

pub struct NativeImageSelection {
    app: AppHandle,
}

impl NativeImageSelection {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Run<SelectImages> for NativeImageSelection {
    type Error = AppError;

    async fn run(&self, operation: &SelectImages) -> AppResult<Option<Vec<PathBuf>>> {
        let app = self.app.clone();

        let kind = operation.kind;

        tokio::task::spawn_blocking(move || choose(&app, kind))
            .await
            .map_err(|_| AppError::RecoveryRequired)?
    }
}

#[cfg(test)]
mod test {
    use crate::implementation::image_selection::folder_sources;

    #[test]
    fn folder_is_flat_and_content_based() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;

        let image = directory.path().join("image.unusual");

        image::DynamicImage::new_rgb8(4, 6).save_with_format(&image, image::ImageFormat::Png)?;

        std::fs::write(directory.path().join("notes.txt"), b"text")?;

        std::fs::create_dir(directory.path().join("nested"))?;

        image::DynamicImage::new_rgb8(4, 6).save(directory.path().join("nested/hidden.png"))?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&image, directory.path().join("linked.png"))?;

        assert_eq!(folder_sources(directory.path())?, vec![image]);

        Ok(())
    }

    #[test]
    fn invalid_supported_extension_is_retained_for_preparation()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;

        let image = directory.path().join("broken.PNG");

        std::fs::write(&image, b"not an image")?;

        assert_eq!(folder_sources(directory.path())?, vec![image]);

        Ok(())
    }
}
