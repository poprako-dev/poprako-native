use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::implementation::image::error::ImageError;
use crate::implementation::resource::ResourceStore;
use crate::implementation::resource::path::{checked_path, valid_id};

pub struct OriginalCandidate {
    pub comic_id: Uuid,
    pub page_id: Uuid,
    pub reference: String,
}

fn owned_directories(
    root: &Path,
    directory: &Path,
) -> Result<Vec<(Uuid, std::path::PathBuf)>, ImageError> {
    let directory = checked_path(root, directory)?;

    let mut result = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let name = entry.file_name();

        let Some(name) = name.to_str() else { continue };

        let Ok(id) = valid_id(name) else { continue };

        let path = entry.path();

        let relative = path
            .strip_prefix(root)
            .map_err(|_| ImageError::InvalidReference)?;

        checked_path(root, relative)?;

        result.push((id, relative.to_path_buf()));
    }

    Ok(result)
}

/// # Errors
/// Returns an error without guessing ownership when a directory cannot be inspected.
pub fn original_candidates(store: &ResourceStore) -> Result<Vec<OriginalCandidate>, ImageError> {
    let mut candidates = Vec::new();

    for (comic_id, comic_path) in owned_directories(store.root(), Path::new("library"))? {
        for (page_id, page_path) in owned_directories(store.root(), &comic_path)? {
            for entry in fs::read_dir(store.root().join(&page_path))? {
                let entry = entry?;

                if !entry.file_type()?.is_file() {
                    continue;
                }

                let path = page_path.join(entry.file_name());

                let Some(reference) = path.to_str() else {
                    continue;
                };

                let reference = reference.replace('\\', "/");

                if store.resolve(&reference, comic_id, page_id).is_err() {
                    continue;
                }

                candidates.push(OriginalCandidate {
                    comic_id,
                    page_id,
                    reference,
                });
            }
        }
    }

    Ok(candidates)
}

/// Only call during successful startup recovery, before any preparation starts.
/// # Errors
/// Stops on cleanup failure and leaves unknown files and names untouched.
pub fn discard_stale_staging(store: &ResourceStore) -> Result<(), ImageError> {
    for (_, task_path) in owned_directories(store.root(), Path::new("staging"))? {
        let task = store.root().join(&task_path);

        for entry in fs::read_dir(&task)? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let name = entry.file_name();

            let Some(name) = name.to_str() else { continue };

            if valid_id(name).is_err() {
                continue;
            }

            let path = checked_path(store.root(), &task_path.join(name))?;

            fs::remove_file(path)?;
        }

        if fs::read_dir(&task)?.next().is_none() {
            fs::remove_dir(task)?;
        }
    }

    Ok(())
}
