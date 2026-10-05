use std::path::Path;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::implementation::coordinator::{preference_migration, shortcut_migration};
use crate::result::{AppError, AppResult};

/// # Errors
/// Returns storage or migration failure; the existing database is retained.
pub async fn open(path: &Path) -> AppResult<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(2));

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;

    if let Err(error) = preference_migration::preflight(&pool).await {
        pool.close().await;

        return Err(error);
    }

    if sqlx::migrate!("./migration").run(&pool).await.is_err() {
        pool.close().await;

        return Err(AppError::Migration);
    }

    if let Err(error) = shortcut_migration::run(&pool).await {
        pool.close().await;

        return Err(error);
    }

    Ok(pool)
}
