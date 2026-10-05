use std::collections::HashSet;

use crate::model::preference::{ApplicationPreference, ShortcutBinding};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

#[must_use]
pub fn bindings_conflict(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    let (
        ShortcutBinding::Chord {
            key: left_key,
            control: left_control,
            meta: left_meta,
            alt: left_alt,
            shift: left_shift,
            ..
        },
        ShortcutBinding::Chord {
            key: right_key,
            control: right_control,
            meta: right_meta,
            alt: right_alt,
            shift: right_shift,
            ..
        },
    ) = (left, right)
    else {
        return false;
    };

    left_key == right_key
        && left_control == right_control
        && left_meta == right_meta
        && left_alt == right_alt
        && left_shift == right_shift
}

/// # Errors
/// Rejects invalid preference values and conflicting shortcuts.
pub fn validate(preference: &ApplicationPreference) -> AppResult<()> {
    if !preference.marker_opacity.is_finite()
        || preference.marker_opacity <= 0.0
        || preference.marker_opacity > 1.0
    {
        return Err(AppError::InvalidInput);
    }

    let mut ids = HashSet::new();

    for character in &preference.special_character {
        if character.text.trim().is_empty() || !ids.insert(&character.id) {
            return Err(AppError::InvalidInput);
        }

        if !matches!(
            character.id.as_str(),
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10"
        ) {
            validate_id(&character.id)?;
        }
    }

    let defaults = ApplicationPreference::default();

    if preference.shortcut.len() != defaults.shortcut.len() {
        return Err(AppError::InvalidInput);
    }

    for expected in &defaults.shortcut {
        if preference
            .shortcut
            .iter()
            .filter(|item| item.action == expected.action)
            .count()
            != 1
        {
            return Err(AppError::InvalidInput);
        }
    }

    for (index, shortcut) in preference.shortcut.iter().enumerate() {
        if let ShortcutBinding::Chord { key, .. } = &shortcut.binding {
            if key.trim().is_empty() || key.len() > 64 {
                return Err(AppError::InvalidInput);
            }

            if preference.shortcut[..index]
                .iter()
                .any(|previous| bindings_conflict(&previous.binding, &shortcut.binding))
            {
                return Err(AppError::Conflict);
            }
        }
    }

    Ok(())
}
