use std::error::Error;

use poprako_orchestra::nucl::{Nucl, NuclError};
use uuid::Uuid;

use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::result::AppError;

#[tokio::test]
async fn connection_policy_and_busy_are_real() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("poprako-policy-{}.sqlite", Uuid::new_v4()));

    let pool = database::open(&path).await?;

    let mut held = Vec::new();

    for _ in 0..4 {
        let mut connection = pool.acquire().await?;

        let journal = sqlx::query_scalar!("SELECT CAST(journal_mode AS TEXT) AS \"journal_mode!: String\" FROM pragma_journal_mode").fetch_one(&mut *connection).await?;

        let foreign_keys = sqlx::query_scalar!("PRAGMA foreign_keys")
            .fetch_one(&mut *connection)
            .await?;

        let synchronous = sqlx::query_scalar!("PRAGMA synchronous")
            .fetch_one(&mut *connection)
            .await?;

        let timeout = sqlx::query_scalar!("PRAGMA busy_timeout")
            .fetch_one(&mut *connection)
            .await?;

        assert_eq!(journal, "wal");

        assert_eq!(foreign_keys, Some(1));

        assert_eq!(synchronous, Some(2));

        assert_eq!(timeout, Some(2000));

        held.push(connection);
    }

    drop(held);

    let transaction = pool.begin_with("BEGIN IMMEDIATE").await?;

    let coordinator = SqliteCoordinator::new(pool.clone());

    let busy = coordinator
        .coord(async |_context| Ok::<_, AppError>(()))
        .await;

    assert!(matches!(busy, Err(NuclError::Backend(AppError::Busy))));

    transaction.rollback().await?;

    coordinator
        .coord(async |_context| Ok::<_, AppError>(()))
        .await?;

    pool.close().await;

    std::fs::remove_file(path)?;

    Ok(())
}

#[tokio::test]
async fn migration_failure_preserves_prior_data() -> Result<(), Box<dyn Error>> {
    let directory = std::env::temp_dir().join(format!("poprako-migration-{}", Uuid::new_v4()));

    std::fs::create_dir_all(&directory)?;

    let path = directory.join("database.sqlite");

    let pool = database::open(&path).await?;

    let id = Uuid::new_v4().to_string();

    sqlx::query!("INSERT INTO comic (id, title, subtitle, author, created_at, updated_at) VALUES (?, 'original', '', '', 1, 1)", id).execute(&pool).await?;

    std::fs::write(
        directory.join("202609260001_initial.sql"),
        include_str!("../../../migration/202609260001_initial.sql"),
    )?;

    std::fs::write(
        directory.join("202609260002_failure.sql"),
        "UPDATE comic SET title = 'changed'; CREATE TABLE comic (id TEXT);",
    )?;

    let migrator = sqlx::migrate::Migrator::new(directory.as_path()).await?;

    assert!(migrator.run(&pool).await.is_err());

    let title = sqlx::query_scalar!("SELECT title FROM comic WHERE id = ?", id)
        .fetch_one(&pool)
        .await?;

    assert_eq!(title, "original");

    pool.close().await;

    std::fs::remove_dir_all(directory)?;

    Ok(())
}
