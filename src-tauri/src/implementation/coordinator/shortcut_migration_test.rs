use std::error::Error;

use crate::implementation::coordinator::database;
use crate::implementation::coordinator::preference_migration_test::database_before_shortcuts;
use crate::model::preference::{
    ApplicationPreference, ShortcutAction, ShortcutBinding, ShortcutScope,
};

fn legacy_preference(macos: bool) -> ApplicationPreference {
    let mut preference = ApplicationPreference::default();

    for shortcut in &mut preference.shortcut {
        let (key, shift, navigation) = match shortcut.action {
            ShortcutAction::Save => ("KeyS", false, false),
            ShortcutAction::CycleMode => ("KeyM", true, false),
            ShortcutAction::ToggleRelocation => ("KeyL", true, false),
            ShortcutAction::PreviousUnit => ("ArrowUp", false, true),
            ShortcutAction::NextUnit => ("ArrowDown", false, true),
            ShortcutAction::PreviousPage => ("ArrowLeft", false, true),
            ShortcutAction::NextPage => ("ArrowRight", false, true),
            _ => {
                shortcut.binding = ShortcutBinding::Unbound;

                continue;
            }
        };

        shortcut.binding = ShortcutBinding::Chord {
            key: key.to_owned(),
            control: !navigation && !macos,
            meta: !navigation && macos,
            alt: navigation,
            shift,
            scope: match navigation {
                true => ShortcutScope::CanvasNavigation,
                false => ShortcutScope::Editor,
            },
        };
    }

    preference.relocation_enabled = true;

    preference.marker_opacity = 0.35;

    preference
}

async fn read(pool: &sqlx::SqlitePool) -> Result<ApplicationPreference, Box<dyn Error>> {
    let payload =
        sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
            .fetch_one(pool)
            .await?;

    Ok(serde_json::from_str(&payload)?)
}

fn legacy_payload(
    preference: &ApplicationPreference,
    light_applied: bool,
) -> Result<String, Box<dyn Error>> {
    let mut value = serde_json::to_value(preference)?;

    if !light_applied {
        let Some(object) = value.as_object_mut() else {
            return Err("preference must be an object".into());
        };

        object.insert(
            "theme".to_owned(),
            serde_json::Value::String("dark".to_owned()),
        );
    }

    Ok(serde_json::to_string(&value)?.replace("toggle_proofread_preview", "cycle_marker_opacity"))
}

#[tokio::test]
async fn mac_and_windows_defaults_upgrade_and_survive_reopening() -> Result<(), Box<dyn Error>> {
    for (macos, light_applied) in [(true, false), (false, false), (true, true), (false, true)] {
        let original = legacy_preference(macos);

        let (directory, path) =
            database_before_shortcuts(&legacy_payload(&original, light_applied)?, light_applied)
                .await?;

        let pool = database::open(&path).await?;

        let saved = read(&pool).await?;

        assert_eq!(saved.shortcut, ApplicationPreference::default().shortcut);

        assert_eq!(saved.special_character, original.special_character);

        assert_eq!(saved.relocation_enabled, original.relocation_enabled);

        assert_eq!(
            saved.marker_opacity.to_bits(),
            original.marker_opacity.to_bits()
        );

        pool.close().await;

        let reopened = database::open(&path).await?;

        assert_eq!(read(&reopened).await?, saved);

        reopened.close().await;

        std::fs::remove_dir_all(directory)?;
    }

    Ok(())
}

#[tokio::test]
async fn custom_bindings_win_conflicts_and_are_never_remigrated() -> Result<(), Box<dyn Error>> {
    let mut original = legacy_preference(true);

    for shortcut in &mut original.shortcut {
        if shortcut.action == ShortcutAction::Save {
            shortcut.binding = ShortcutBinding::Chord {
                key: "KeyX".to_owned(),
                control: true,
                meta: false,
                alt: false,
                shift: false,
                scope: ShortcutScope::Editor,
            };
        }

        if shortcut.action == ShortcutAction::CycleMode {
            shortcut.binding = ShortcutBinding::Unbound;
        }
    }

    let (directory, path) =
        database_before_shortcuts(&legacy_payload(&original, true)?, true).await?;

    let pool = database::open(&path).await?;

    let mut saved = read(&pool).await?;

    for action in [
        ShortcutAction::Save,
        ShortcutAction::CycleMode,
        ShortcutAction::ToggleProofreadPreview,
    ] {
        assert_eq!(
            saved.shortcut.iter().find(|item| item.action == action),
            original.shortcut.iter().find(|item| item.action == action)
        );
    }

    saved.shortcut = original.shortcut;

    let payload = serde_json::to_string(&saved)?;

    sqlx::query!(
        "UPDATE application_preference SET payload = ? WHERE singleton = 1",
        payload
    )
    .execute(&pool)
    .await?;

    pool.close().await;

    let reopened = database::open(&path).await?;

    assert_eq!(read(&reopened).await?, saved);

    reopened.close().await;

    std::fs::remove_dir_all(directory)?;

    Ok(())
}

#[test]
fn defaults_match_the_web_on_every_platform() {
    let expected = [
        (ShortcutAction::CycleMode, "KeyM", true, false, false),
        (ShortcutAction::ToggleRelocation, "KeyL", true, false, false),
        (
            ShortcutAction::ToggleProofreadPreview,
            "KeyX",
            true,
            false,
            false,
        ),
        (ShortcutAction::NextUnit, "Tab", false, false, false),
        (ShortcutAction::PreviousUnit, "Tab", false, false, true),
        (ShortcutAction::PreviousPage, "KeyU", true, false, false),
        (ShortcutAction::NextPage, "KeyD", true, false, false),
        (
            ShortcutAction::InsertRecentSymbol,
            "KeyQ",
            true,
            false,
            false,
        ),
        (
            ShortcutAction::InsertFavoriteOne,
            "Digit1",
            false,
            true,
            false,
        ),
        (
            ShortcutAction::InsertFavoriteTwo,
            "Digit2",
            false,
            true,
            false,
        ),
        (
            ShortcutAction::InsertFavoriteThree,
            "Digit3",
            false,
            true,
            false,
        ),
        (ShortcutAction::Save, "KeyS", true, false, false),
    ];

    let defaults = ApplicationPreference::default();

    assert_eq!(defaults.shortcut.len(), expected.len());

    for (shortcut, (action, key, control, alt, shift)) in defaults.shortcut.iter().zip(expected) {
        assert_eq!(shortcut.action, action);

        assert_eq!(
            shortcut.binding,
            ShortcutBinding::Chord {
                key: key.to_owned(),
                control,
                meta: false,
                alt,
                shift,
                scope: ShortcutScope::Editor,
            }
        );
    }

    assert!(crate::complex::preference::validate(&defaults).is_ok());
}
