use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rules;

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;

        if ["target", "gen", "generated"].contains(&entry.file_name().to_string_lossy().as_ref()) {
            continue;
        }

        let path = entry.path();

        let kind = entry.file_type()?;

        if kind.is_dir() {
            collect(&path, files)?;
        }

        if kind.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    Ok(())
}

/// # Errors
/// Reports unreadable directories or source files, and invalid Rust syntax.
pub fn check(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();

    collect(root, &mut files)?;

    files.sort();

    let mut errors = Vec::new();

    for path in files {
        let text = fs::read_to_string(&path)?;

        let lines = text.lines().count();

        if lines > 400 {
            errors.push(format!(
                "{}: {lines} physical lines exceeds 400",
                path.display()
            ));
        }

        if path.file_name().is_some_and(|name| name == "mod.rs") {
            errors.push(format!("{}: use sibling module file", path.display()));
        }

        let parsed = syn::parse_file(&text)?;

        errors.extend(
            rules::check(&parsed)
                .into_iter()
                .map(|message| format!("{}: {message}", path.display())),
        );
    }

    Ok(errors)
}

#[cfg(test)]
mod test;
