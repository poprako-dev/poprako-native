use std::collections::HashSet;

use crate::data::search::Replacement;
use crate::result::{AppError, AppResult};
use crate::value::validation::normalize_body;

/// Applies independent rules against the original bytes, never against earlier replacements.
/// # Errors
/// Rejects missing, duplicate, or overlapping rule origins and invalid rule counts.
pub fn transform(text: &str, rules: &[Replacement]) -> AppResult<String> {
    if rules.is_empty() || rules.len() > 20 {
        return Err(AppError::InvalidInput);
    }

    let mut origins = HashSet::new();

    let mut matches = Vec::new();

    for rule in rules {
        if rule.origin.is_empty() || !origins.insert(&rule.origin) {
            return Err(AppError::InvalidInput);
        }

        matches.extend(
            text.match_indices(&rule.origin)
                .map(|(start, matched)| (start, start + matched.len(), rule.target.as_str())),
        );
    }

    matches.sort_unstable_by_key(|(start, end, _)| (*start, *end));

    let mut result = String::new();

    let mut cursor = 0;

    for (start, end, replacement) in matches {
        if start < cursor {
            return Err(AppError::InvalidInput);
        }

        result.push_str(&text[cursor..start]);

        result.push_str(replacement);

        cursor = end;
    }

    result.push_str(&text[cursor..]);

    Ok(normalize_body(&result))
}
