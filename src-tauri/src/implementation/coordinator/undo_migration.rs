use sqlx::SqlitePool;

use crate::complex::preference::{bindings_conflict, validate};
use crate::model::preference::{
    ApplicationPreference, Shortcut, ShortcutAction, ShortcutBinding, undo_shortcut,
};
use crate::result::{AppError, AppResult};

/// # Errors
/// Validates exactly the pre-undo action set without accepting a partial current preference.
pub fn validate_legacy(preference: &ApplicationPreference) -> AppResult<()> {
    if preference
        .shortcut
        .iter()
        .any(|item| item.action == ShortcutAction::Undo)
    {
        return Err(AppError::RecoveryRequired);
    }

    let mut current = preference.clone();

    current.shortcut.push(Shortcut {
        action: ShortcutAction::Undo,
        binding: ShortcutBinding::Unbound,
    });

    validate(&current).map_err(|_| AppError::RecoveryRequired)
}

/// # Errors
/// Reads the upgrade generation, including databases predating the upgrade ledger.
pub async fn completed(pool: &SqlitePool) -> AppResult<bool> {
    let exists = sqlx::query_scalar!("SELECT COUNT(*) AS \"count!: i64\" FROM sqlite_schema WHERE type = 'table' AND name = 'application_preference_upgrade'").fetch_one(pool).await?;

    if exists == 0 {
        return Ok(false);
    }

    let done = sqlx::query_scalar!("SELECT COUNT(*) AS \"count!: i64\" FROM application_preference_upgrade WHERE name = 'undo_shortcut'").fetch_one(pool).await?;

    Ok(done > 0)
}

/// # Errors
/// Adds the undo binding atomically; existing custom bindings always win conflicts.
pub async fn run(pool: &SqlitePool) -> AppResult<()> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;

    let done = sqlx::query_scalar!("SELECT COUNT(*) AS \"count!: i64\" FROM application_preference_upgrade WHERE name = 'undo_shortcut'").fetch_one(&mut *transaction).await?;

    if done > 0 {
        transaction.rollback().await?;

        return Ok(());
    }

    let payload =
        sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
            .fetch_optional(&mut *transaction)
            .await?;

    if let Some(payload) = payload {
        let mut preference: ApplicationPreference =
            serde_json::from_str(&payload).map_err(|_| AppError::RecoveryRequired)?;

        validate_legacy(&preference)?;

        let mut undo = undo_shortcut(cfg!(target_os = "macos"));

        if preference
            .shortcut
            .iter()
            .any(|item| bindings_conflict(&item.binding, &undo.binding))
        {
            undo.binding = ShortcutBinding::Unbound;
        }

        preference.shortcut.insert(0, undo);

        validate(&preference).map_err(|_| AppError::RecoveryRequired)?;

        let upgraded =
            serde_json::to_string(&preference).map_err(|_| AppError::RecoveryRequired)?;

        sqlx::query!(
            "UPDATE application_preference SET payload = ? WHERE singleton = 1",
            upgraded
        )
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query!("INSERT INTO application_preference_upgrade (name) VALUES ('undo_shortcut')")
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}
