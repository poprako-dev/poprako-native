use std::error::Error;
use std::path::PathBuf;

use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::implementation::coordinator::preference_migration_test::database_before_shortcuts;
use crate::implementation::coordinator::{database, shortcut_migration, undo_migration};
use crate::model::preference::{
    ApplicationPreference, ShortcutAction, ShortcutBinding, ShortcutScope, undo_shortcut,
};

fn legacy() -> ApplicationPreference {
    let mut preference = ApplicationPreference::default();

    preference
        .shortcut
        .retain(|item| item.action != ShortcutAction::Undo);

    preference
}

async fn read(pool: &SqlitePool) -> Result<ApplicationPreference, Box<dyn Error>> {
    let payload =
        sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
            .fetch_one(pool)
            .await?;

    Ok(serde_json::from_str(&payload)?)
}

async fn write(
    pool: &SqlitePool,
    preference: &ApplicationPreference,
) -> Result<(), Box<dyn Error>> {
    let payload = serde_json::to_string(preference)?;

    sqlx::query!(
        "UPDATE application_preference SET payload = ? WHERE singleton = 1",
        payload
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn before_undo(
    preference: &ApplicationPreference,
) -> Result<(PathBuf, PathBuf, SqlitePool), Box<dyn Error>> {
    let (directory, path) =
        database_before_shortcuts(&serde_json::to_string(preference)?, true).await?;

    let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(&path)).await?;

    sqlx::migrate!("./migration").run(&pool).await?;

    shortcut_migration::run(&pool).await?;

    Ok((directory, path, pool))
}

#[test]
fn undo_defaults_use_only_the_platform_modifier() {
    for macos in [false, true] {
        assert_eq!(
            undo_shortcut(macos).binding,
            ShortcutBinding::Chord {
                key: "Period".to_owned(),
                control: !macos,
                meta: macos,
                alt: false,
                shift: false,
                scope: ShortcutScope::Editor,
            }
        );
    }
}

#[tokio::test]
async fn undo_migration_preserves_custom_settings_and_unbinding() -> Result<(), Box<dyn Error>> {
    let mut original = legacy();

    let Some(save) = original
        .shortcut
        .iter_mut()
        .find(|item| item.action == ShortcutAction::Save)
    else {
        return Err("missing save shortcut".into());
    };

    save.binding = undo_shortcut(cfg!(target_os = "macos")).binding;

    original.special_character.reverse();

    original.marker_opacity = 0.4;

    let (directory, path, pool) = before_undo(&original).await?;

    pool.close().await;

    let pool = database::open(&path).await?;

    let saved = read(&pool).await?;

    assert_eq!(saved.shortcut[0].action, ShortcutAction::Undo);

    assert_eq!(saved.shortcut[0].binding, ShortcutBinding::Unbound);

    assert_eq!(&saved.shortcut[1..], original.shortcut.as_slice());

    assert_eq!(saved.special_character, original.special_character);

    assert_eq!(
        saved.marker_opacity.to_bits(),
        original.marker_opacity.to_bits()
    );

    for binding in [
        ShortcutBinding::Chord {
            key: "Period".to_owned(),
            control: false,
            meta: false,
            alt: true,
            shift: false,
            scope: ShortcutScope::Editor,
        },
        ShortcutBinding::Unbound,
    ] {
        let mut customized = saved.clone();

        customized.shortcut[0].binding = binding;

        write(&pool, &customized).await?;

        let reopened = database::open(&path).await?;

        assert_eq!(read(&reopened).await?, customized);

        reopened.close().await;
    }

    pool.close().await;

    std::fs::remove_dir_all(directory)?;

    Ok(())
}

#[tokio::test]
async fn undo_migration_rolls_back_payload_when_marker_fails() -> Result<(), Box<dyn Error>> {
    let original = legacy();

    let (directory, path, pool) = before_undo(&original).await?;

    sqlx::raw_sql("CREATE TRIGGER fail_undo BEFORE INSERT ON application_preference_upgrade WHEN NEW.name = 'undo_shortcut' BEGIN SELECT RAISE(ABORT, 'injected failure'); END;").execute(&pool).await?;

    assert!(database::open(&path).await.is_err());

    assert_eq!(read(&pool).await?, original);

    assert!(!undo_migration::completed(&pool).await?);

    sqlx::raw_sql("DROP TRIGGER fail_undo")
        .execute(&pool)
        .await?;

    pool.close().await;

    let reopened = database::open(&path).await?;

    assert_eq!(read(&reopened).await?, ApplicationPreference::default());

    reopened.close().await;

    std::fs::remove_dir_all(directory)?;

    Ok(())
}

#[tokio::test]
async fn undo_migration_rejects_incomplete_or_duplicate_actions_without_reset()
-> Result<(), Box<dyn Error>> {
    let original = legacy();

    let (directory, path, pool) = before_undo(&original).await?;

    let mut missing = original.clone();

    missing.shortcut.pop();

    let mut duplicate = original.clone();

    duplicate.shortcut[1] = duplicate.shortcut[0].clone();

    for invalid in [missing, duplicate, ApplicationPreference::default()] {
        write(&pool, &invalid).await?;

        assert!(database::open(&path).await.is_err());

        assert_eq!(read(&pool).await?, invalid);
    }

    write(&pool, &original).await?;

    undo_migration::run(&pool).await?;

    write(&pool, &original).await?;

    assert!(database::open(&path).await.is_err());

    assert_eq!(read(&pool).await?, original);

    pool.close().await;

    std::fs::remove_dir_all(directory)?;

    Ok(())
}
