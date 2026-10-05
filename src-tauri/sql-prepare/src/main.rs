use std::error::Error;
use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("database path argument is required")?,
    );

    if path.exists() {
        return Err("refusing to overwrite an existing database".into());
    }

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    sqlx::migrate!("../migration").run(&pool).await?;

    pool.close().await;

    Ok(())
}
