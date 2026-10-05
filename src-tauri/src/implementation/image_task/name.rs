use std::cmp::Ordering;
use std::path::PathBuf;

use unicode_normalization::UnicodeNormalization;

use crate::result::{AppError, AppResult};

fn compare_normalized(left: &str, right: &str) -> Ordering {
    let mut left = left.chars().peekable();

    let mut right = right.chars().peekable();

    loop {
        let (Some(&a), Some(&b)) = (left.peek(), right.peek()) else {
            return left.next().cmp(&right.next());
        };

        if a.is_ascii_digit() && b.is_ascii_digit() {
            let a: String = std::iter::from_fn(|| left.next_if(char::is_ascii_digit)).collect();

            let b: String = std::iter::from_fn(|| right.next_if(char::is_ascii_digit)).collect();

            let significant_a = a.trim_start_matches('0');

            let significant_b = b.trim_start_matches('0');

            let order = significant_a
                .len()
                .cmp(&significant_b.len())
                .then_with(|| significant_a.cmp(significant_b))
                .then_with(|| a.len().cmp(&b.len()));

            if order != Ordering::Equal {
                return order;
            }

            continue;
        }

        let order = a.cmp(&b);

        if order != Ordering::Equal {
            return order;
        }

        left.next();

        right.next();
    }
}

/// # Errors
/// Rejects non-Unicode filenames instead of silently changing ordering.
pub fn sort_sources(paths: Vec<PathBuf>) -> AppResult<Vec<(PathBuf, String)>> {
    let mut sources = paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(AppError::InvalidInput)?
                .to_owned();

            let normalized = name.nfc().collect::<String>().to_lowercase();

            Ok((path, name, normalized))
        })
        .collect::<AppResult<Vec<_>>>()?;

    sources.sort_by(|left, right| {
        compare_normalized(&left.2, &right.2)
            .then_with(|| left.1.as_bytes().cmp(right.1.as_bytes()))
    });

    Ok(sources
        .into_iter()
        .map(|(path, name, _)| (path, name))
        .collect())
}
