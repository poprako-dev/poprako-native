use std::fs;
use std::path::{Component, Path, PathBuf};

use uuid::Uuid;

use crate::implementation::image::error::ImageError;

/// # Errors
/// Returns an error when validation, resource access, or image processing fails.
pub fn valid_id(value: &str) -> Result<Uuid, ImageError> {
    let id = Uuid::parse_str(value).map_err(|_| ImageError::InvalidReference)?;

    if id.get_version_num() != 4 || id.to_string() != value {
        return Err(ImageError::InvalidReference);
    }

    Ok(id)
}

/// # Errors
/// Requires a canonical immutable original reference scoped to its owning page.
pub fn original_reference(
    reference: &str,
    comic_id: Uuid,
    page_id: Uuid,
) -> Result<PathBuf, ImageError> {
    valid_id(&comic_id.to_string())?;

    valid_id(&page_id.to_string())?;

    let parts: Vec<_> = reference.split('/').collect();

    if parts.len() != 4
        || parts[0] != "library"
        || parts[1] != comic_id.to_string()
        || parts[2] != page_id.to_string()
    {
        return Err(ImageError::InvalidReference);
    }

    let (image_id, extension) = parts[3]
        .rsplit_once('.')
        .ok_or(ImageError::InvalidReference)?;

    valid_id(image_id)?;

    if !matches!(extension, "jpg" | "png" | "webp" | "bmp") {
        return Err(ImageError::InvalidReference);
    }

    Ok(PathBuf::from(reference))
}

fn inspect_path(
    root: &Path,
    relative: &Path,
    missing_ok: bool,
) -> Result<Option<PathBuf>, ImageError> {
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ImageError::InvalidReference);
    }

    let mut path = root.to_path_buf();

    for component in relative.components() {
        let Component::Normal(segment) = component else {
            return Err(ImageError::InvalidReference);
        };

        path.push(segment);

        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if missing_ok && error.kind() == std::io::ErrorKind::NotFound => {
                let parent = path.parent().ok_or(ImageError::InvalidReference)?;

                if !parent.canonicalize()?.starts_with(root) {
                    return Err(ImageError::InvalidReference);
                }

                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };

        if metadata.file_type().is_symlink() {
            return Err(ImageError::InvalidReference);
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;

            if metadata.file_attributes() & 0x400 != 0 {
                return Err(ImageError::InvalidReference);
            }
        }
    }

    if !path.canonicalize()?.starts_with(root) {
        return Err(ImageError::InvalidReference);
    }

    Ok(Some(path))
}

/// # Errors
/// Returns an error when validation, resource access, or image processing fails.
pub fn checked_path(root: &Path, relative: &Path) -> Result<PathBuf, ImageError> {
    inspect_path(root, relative, false)?.ok_or(ImageError::InvalidReference)
}

/// # Errors
/// A missing entry is finished cleanup only after every existing ancestor was checked.
pub fn cleanup_path(root: &Path, relative: &Path) -> Result<Option<PathBuf>, ImageError> {
    inspect_path(root, relative, true)
}
