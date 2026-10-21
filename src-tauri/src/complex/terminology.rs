use std::collections::HashSet;

use crate::data::terminology::{TermMetadata, TermbaseMetadata, TerminologyQuery};
use crate::result::{AppError, AppResult};

pub mod import;

#[cfg(test)]
mod import_test;

#[must_use]
pub fn key(value: &str) -> String {
    value.trim().to_lowercase()
}

/// # Errors
/// Rejects blank names; Rust owns the Unicode whitespace normalization.
pub fn normalize_base(input: &TermbaseMetadata) -> AppResult<TermbaseMetadata> {
    let name = input.name.trim().to_owned();

    if name.is_empty() {
        return Err(AppError::InvalidInput);
    }

    Ok(TermbaseMetadata {
        name,
        description: input.description.trim().to_owned(),
    })
}

/// # Errors
/// Rejects empty sources, missing translations, and case-insensitive duplicate translations.
pub fn normalize_term(input: TermMetadata) -> AppResult<TermMetadata> {
    let source = input.source.trim().to_owned();

    if source.is_empty() || input.targets.is_empty() {
        return Err(AppError::InvalidInput);
    }

    let mut seen = HashSet::new();

    let mut targets = Vec::new();

    for target in input.targets {
        let value = target.trim().to_owned();

        if value.is_empty() || !seen.insert(key(&value)) {
            return Err(AppError::InvalidInput);
        }

        targets.push(value);
    }

    Ok(TermMetadata {
        source,
        targets,
        comment: input.comment.trim().to_owned(),
    })
}

/// # Errors
/// Bounds individual reads without imposing a library-size limit.
pub fn validate_query(query: &TerminologyQuery) -> AppResult<()> {
    if query.limit == 0 || query.limit > 100 {
        return Err(AppError::InvalidInput);
    }

    Ok(())
}
