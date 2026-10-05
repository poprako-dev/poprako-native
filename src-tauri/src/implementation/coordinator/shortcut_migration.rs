use sqlx::SqlitePool;

use crate::complex::preference::{bindings_conflict, validate};
use crate::model::preference::{
    ApplicationPreference, Shortcut, ShortcutAction, ShortcutBinding, ShortcutScope,
};
use crate::result::{AppError, AppResult};

fn was_default(shortcut: &Shortcut) -> bool {
    let (key, shift, navigation) = match shortcut.action {
        ShortcutAction::Save => ("KeyS", false, false),
        ShortcutAction::CycleMode => ("KeyM", true, false),
        ShortcutAction::ToggleRelocation => ("KeyL", true, false),
        ShortcutAction::PreviousUnit => ("ArrowUp", false, true),
        ShortcutAction::NextUnit => ("ArrowDown", false, true),
        ShortcutAction::PreviousPage => ("ArrowLeft", false, true),
        ShortcutAction::NextPage => ("ArrowRight", false, true),
        _ => return shortcut.binding == ShortcutBinding::Unbound,
    };

    [false, true].into_iter().any(|macos| {
        shortcut.binding
            == ShortcutBinding::Chord {
                key: key.to_owned(),
                control: !navigation && !macos,
                meta: !navigation && macos,
                alt: navigation,
                shift,
                scope: match navigation {
                    true => ShortcutScope::CanvasNavigation,
                    false => ShortcutScope::Editor,
                },
            }
    })
}

fn upgrade(preference: &mut ApplicationPreference) {
    let customized: Vec<_> = preference
        .shortcut
        .iter()
        .filter(|item| !was_default(item))
        .cloned()
        .collect();

    preference.shortcut = ApplicationPreference::default()
        .shortcut
        .into_iter()
        .map(|default| {
            let existing = preference
                .shortcut
                .iter()
                .find(|item| item.action == default.action);

            match existing {
                Some(item) if !was_default(item) => item.clone(),
                Some(item)
                    if customized
                        .iter()
                        .any(|custom| bindings_conflict(&custom.binding, &default.binding)) =>
                {
                    item.clone()
                }
                _ => default,
            }
        })
        .collect();

    for shortcut in &mut preference.shortcut {
        if let ShortcutBinding::Chord { scope, .. } = &mut shortcut.binding {
            *scope = ShortcutScope::Editor;
        }
    }
}

/// # Errors
/// Atomically upgrades recognized defaults, preserving custom bindings and their conflicts.
pub async fn run(pool: &SqlitePool) -> AppResult<()> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;

    let done = sqlx::query_scalar!("SELECT COUNT(*) AS \"count!: i64\" FROM application_preference_upgrade WHERE name = 'web_shortcuts'").fetch_one(&mut *transaction).await?;

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

        upgrade(&mut preference);

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

    sqlx::query!("INSERT INTO application_preference_upgrade (name) VALUES ('web_shortcuts')")
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}
