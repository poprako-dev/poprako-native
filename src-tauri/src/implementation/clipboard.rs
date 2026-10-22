use crate::result::{AppError, AppResult};

/// # Errors
/// Copies plain text through the native clipboard without granting frontend clipboard access.
pub fn write(text: String) -> AppResult<()> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text))
        .map_err(|_| AppError::Clipboard)
}
