use serde::Deserialize;
use sqlx::SqlitePool;

use crate::complex::preference::validate;
use crate::model::preference::{ApplicationPreference, Shortcut, SpecialCharacter};
use crate::result::{AppError, AppResult};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyTheme {
    System,
    Light,
    Dark,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyPreference {
    #[serde(rename = "theme")]
    _theme: LegacyTheme,
    shortcut: Vec<Shortcut>,
    special_character: Vec<SpecialCharacter>,
    relocation_enabled: bool,
    marker_opacity: f64,
}

fn parse(payload: &str) -> AppResult<ApplicationPreference> {
    if let Ok(current) = serde_json::from_str::<ApplicationPreference>(payload) {
        return Ok(current);
    }

    let legacy: LegacyPreference =
        serde_json::from_str(payload).map_err(|_| AppError::RecoveryRequired)?;

    Ok(ApplicationPreference {
        shortcut: legacy.shortcut,
        special_character: legacy.special_character,
        relocation_enabled: legacy.relocation_enabled,
        marker_opacity: legacy.marker_opacity,
    })
}

/// # Errors
/// Validates the complete old preference before a migration removes its theme.
pub async fn preflight(pool: &SqlitePool) -> AppResult<()> {
    let exists = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!: i64\" FROM sqlite_schema WHERE type = 'table' AND name = 'application_preference'"
    ).fetch_one(pool).await?;

    if exists == 0 {
        return Ok(());
    }

    let payload =
        sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
            .fetch_optional(pool)
            .await?;

    let Some(payload) = payload else {
        return Ok(());
    };

    validate(&parse(&payload)?).map_err(|_| AppError::RecoveryRequired)
}
