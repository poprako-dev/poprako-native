use poprako_orchestra::{Run, Step};
use sqlx::SqliteConnection;

use crate::complex::preference::validate;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::model::preference::ApplicationPreference;
use crate::part::repository::operation::{PersistPreference, ReadPreference};
use crate::result::{AppError, AppResult};

async fn read(connection: &mut SqliteConnection) -> AppResult<ApplicationPreference> {
    let row = sqlx::query!("SELECT payload FROM application_preference WHERE singleton = 1")
        .fetch_optional(connection)
        .await?;

    let Some(row) = row else {
        return Ok(ApplicationPreference::default());
    };

    let preference = serde_json::from_str(&row.payload).map_err(|_| AppError::RecoveryRequired)?;

    validate(&preference).map_err(|_| AppError::RecoveryRequired)?;

    Ok(preference)
}

impl Run<ReadPreference> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, _operation: &ReadPreference) -> AppResult<ApplicationPreference> {
        let mut connection = self.pool.acquire().await?;

        read(&mut connection).await
    }
}

impl Step<PersistPreference, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &PersistPreference,
    ) -> AppResult<ApplicationPreference> {
        validate(&operation.preference)?;

        let existing = read(&mut context.0).await?;

        if existing != operation.baseline {
            return Err(AppError::Conflict);
        }

        if existing == operation.preference {
            return Ok(existing);
        }

        let payload =
            serde_json::to_string(&operation.preference).map_err(|_| AppError::InvalidInput)?;

        sqlx::query!("INSERT INTO application_preference (singleton, payload) VALUES (1, ?) ON CONFLICT(singleton) DO UPDATE SET payload = excluded.payload", payload)
            .execute(&mut *context.0).await?;

        Ok(operation.preference.clone())
    }
}
