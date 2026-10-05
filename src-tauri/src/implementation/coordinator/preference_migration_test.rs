use std::error::Error;
use std::path::PathBuf;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use uuid::Uuid;

use crate::implementation::coordinator::database;
use crate::model::preference::ApplicationPreference;

pub async fn database_before_shortcuts(
    payload: &str,
    light_applied: bool,
) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let directory = std::env::temp_dir().join(format!("poprako-preference-{}", Uuid::new_v4()));

    let migration = directory.join("migration");

    std::fs::create_dir_all(&migration)?;

    std::fs::write(
        migration.join("202609260001_initial.sql"),
        include_str!("../../../migration/202609260001_initial.sql"),
    )?;

    if light_applied {
        std::fs::write(
            migration.join("202610030001_light_appearance.sql"),
            include_str!("../../../migration/202610030001_light_appearance.sql"),
        )?;
    }

    let path = directory.join("database.sqlite");

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    sqlx::migrate::Migrator::new(migration.as_path())
        .await?
        .run(&pool)
        .await?;

    sqlx::query!(
        "INSERT INTO application_preference (singleton, payload) VALUES (1, ?)",
        payload
    )
    .execute(&pool)
    .await?;

    pool.close().await;

    Ok((directory, path))
}

pub async fn legacy_database(payload: &str) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    database_before_shortcuts(payload, false).await
}

fn legacy_payload(theme: &str) -> Result<(ApplicationPreference, String), Box<dyn Error>> {
    let preference = ApplicationPreference {
        relocation_enabled: true,
        marker_opacity: 0.35,
        ..ApplicationPreference::default()
    };

    let mut json = serde_json::to_value(&preference)?;

    let Some(object) = json.as_object_mut() else {
        return Err("preference must serialize as an object".into());
    };

    object.insert(
        "theme".to_owned(),
        serde_json::Value::String(theme.to_owned()),
    );

    Ok((preference, serde_json::to_string(&json)?))
}

#[tokio::test]
async fn all_old_themes_preserve_the_remaining_preference() -> Result<(), Box<dyn Error>> {
    for theme in ["system", "light", "dark"] {
        let (expected, payload) = legacy_payload(theme)?;

        let (directory, path) = legacy_database(&payload).await?;

        let pool = database::open(&path).await?;

        let saved =
            sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
                .fetch_one(&pool)
                .await?;

        assert_eq!(
            serde_json::from_str::<ApplicationPreference>(&saved)?,
            expected
        );

        assert!(!saved.contains("\"theme\""));

        pool.close().await;

        let reopened = database::open(&path).await?;

        reopened.close().await;

        std::fs::remove_dir_all(directory)?;
    }

    Ok(())
}

#[tokio::test]
async fn malformed_preferences_are_retained_before_migration() -> Result<(), Box<dyn Error>> {
    let (_, valid) = legacy_payload("dark")?;

    let cases = [
        "{broken".to_owned(),
        valid.replace("\"dark\"", "\"unknown\""),
        valid.replace("0.35", "0.0"),
        valid.replacen('{', "{\"unexpected\":true,", 1),
        valid.replacen('{', "{\"theme\":\"light\",", 1),
    ];

    for payload in cases {
        let (directory, path) = legacy_database(&payload).await?;

        assert!(database::open(&path).await.is_err());

        let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(&path)).await?;

        let saved =
            sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
                .fetch_one(&pool)
                .await?;

        assert_eq!(saved, payload);

        pool.close().await;

        std::fs::remove_dir_all(directory)?;
    }

    Ok(())
}
