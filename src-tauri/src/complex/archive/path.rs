use unicode_normalization::UnicodeNormalization;

use crate::result::{AppError, AppResult};

/// # Errors
/// Rejects ambiguous names before any entry reaches the filesystem.
pub fn logical_path(name: &str) -> AppResult<String> {
    if name.is_empty() || name.starts_with(['/', '\\']) || name.contains([':', '\0', '\\']) {
        return Err(AppError::InvalidInput);
    }

    let trimmed = name.trim_end_matches('/');

    if trimmed
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == ".." || part.ends_with([' ', '.']))
    {
        return Err(AppError::InvalidInput);
    }

    Ok(trimmed.nfc().collect::<String>().to_lowercase())
}
