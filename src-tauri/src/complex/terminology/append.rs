use std::collections::HashSet;

use crate::complex::terminology::import::{normalize_terms, parse_document};
use crate::complex::terminology::{key, normalize_term};
use crate::data::terminology::portable::PortableTermbaseAppend;
use crate::data::terminology::{TermMetadata, TermbaseAppend};
use crate::result::{AppError, AppResult};

/// # Errors
/// Accepts only a terms envelope; library metadata is never part of an append.
pub fn parse(content: &[u8]) -> AppResult<TermbaseAppend> {
    let input: PortableTermbaseAppend = parse_document(content)?;

    let terms = input
        .terms
        .into_iter()
        .map(|term| TermMetadata {
            source: term.source,
            targets: term.targets,
            comment: term.comment.unwrap_or_default(),
        })
        .collect();

    Ok(TermbaseAppend {
        terms: normalize_terms(terms).map_err(|_| AppError::TermbaseImport)?,
    })
}

/// # Errors
/// Retains the preferred translation and treats comments as complete text blocks.
pub fn merge(existing: TermMetadata, incoming: TermMetadata) -> AppResult<TermMetadata> {
    let mut result = normalize_term(existing)?;

    let incoming = normalize_term(incoming)?;

    if key(&result.source) != key(&incoming.source) {
        return Err(AppError::InvalidInput);
    }

    let mut targets: HashSet<String> = result.targets.iter().map(|value| key(value)).collect();

    for target in incoming.targets {
        if targets.insert(key(&target)) {
            result.targets.push(target);
        }
    }

    if incoming.comment.is_empty() || result.comment == incoming.comment {
        return Ok(result);
    }

    if result.comment.is_empty() {
        result.comment = incoming.comment;

        return Ok(result);
    }

    result.comment.push('\n');

    result.comment.push_str(&incoming.comment);

    Ok(result)
}

#[cfg(test)]
mod test;
