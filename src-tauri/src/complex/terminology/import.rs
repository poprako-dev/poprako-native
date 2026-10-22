use std::collections::HashSet;

use crate::complex::terminology::{key, normalize_base, normalize_term};
use crate::data::terminology::portable::PortableTermbase;
use crate::data::terminology::{TermMetadata, TermbaseImport, TermbaseMetadata};
use crate::result::{AppError, AppResult};

pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// # Errors
/// Validates all entries before creating a library; duplicate sources never overwrite terms.
pub fn normalize(input: TermbaseImport) -> AppResult<TermbaseImport> {
    let metadata = normalize_base(&input.metadata)?;

    let mut sources = HashSet::new();

    let mut terms = Vec::with_capacity(input.terms.len());

    for term in input.terms {
        let term = normalize_term(term)?;

        if !sources.insert(key(&term.source)) {
            return Err(AppError::DuplicateTerm);
        }

        terms.push(term);
    }

    Ok(TermbaseImport { metadata, terms })
}

/// # Errors
/// Accepts the Web portable JSON format, including omitted or null optional text fields.
pub fn parse(content: &[u8]) -> AppResult<TermbaseImport> {
    if u64::try_from(content.len()).map_err(|_| AppError::TermbaseImport)? > MAX_FILE_BYTES {
        return Err(AppError::TermbaseImport);
    }

    let content = content.strip_prefix(b"\xef\xbb\xbf").unwrap_or(content);

    let input: PortableTermbase =
        serde_json::from_slice(content).map_err(|_| AppError::TermbaseImport)?;

    normalize(TermbaseImport {
        metadata: TermbaseMetadata {
            name: input.name,
            description: input.description.unwrap_or_default(),
        },
        terms: input
            .terms
            .into_iter()
            .map(|term| TermMetadata {
                source: term.source,
                targets: term.targets,
                comment: term.comment.unwrap_or_default(),
            })
            .collect(),
    })
    .map_err(|_| AppError::TermbaseImport)
}
