use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::result::{AppError, AppResult};

pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// # Errors
/// Returns an error when the value cannot be represented by the local model.
pub fn validate_id(value: &str) -> AppResult<()> {
    let parsed = Uuid::parse_str(value).map_err(|_| AppError::InvalidInput)?;

    if parsed.get_version_num() != 4
        || parsed.get_variant() != uuid::Variant::RFC4122
        || parsed.hyphenated().to_string() != value
    {
        return Err(AppError::InvalidInput);
    }

    Ok(())
}

/// # Errors
/// Returns an error when the value cannot be represented by the local model.
pub fn timestamp() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Storage)?;

    let value = i64::try_from(duration.as_millis()).map_err(|_| AppError::Storage)?;

    if value > MAX_SAFE_INTEGER {
        return Err(AppError::Storage);
    }

    Ok(value)
}

#[must_use]
pub fn normalize_body(value: &str) -> String {
    if value.chars().all(char::is_whitespace) {
        return String::new();
    }

    value.to_owned()
}

/// # Errors
/// Rejects non-finite or out-of-range normalized coordinates.
pub fn validate_coordinate(value: f64) -> AppResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(AppError::InvalidInput);
    }

    Ok(())
}

/// # Errors
/// Returns an error when the value cannot be represented by the local model.
pub fn position(value: usize) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| AppError::InvalidInput)
}
