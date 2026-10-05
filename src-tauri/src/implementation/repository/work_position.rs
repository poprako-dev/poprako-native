use poprako_orchestra::Step;
use sqlx::SqliteConnection;

use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::model::work_position::{EditorMode, WorkPosition};
use crate::part::repository::operation::PersistWorkPosition;
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn read_position(
    connection: &mut SqliteConnection,
    comic_id: &str,
) -> AppResult<Option<WorkPosition>> {
    let row = sqlx::query!("SELECT comic_id, last_opened_at, page_id, unit_id, mode FROM work_position WHERE comic_id = ?", comic_id)
        .fetch_optional(connection).await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let mode = match row.mode.as_str() {
        "translation" => EditorMode::Translation,
        "proofreading" => EditorMode::Proofreading,
        "readonly" => EditorMode::Readonly,
        _ => return Err(AppError::RecoveryRequired),
    };

    Ok(Some(WorkPosition {
        comic_id: row.comic_id,
        last_opened_at: row.last_opened_at,
        page_id: row.page_id,
        unit_id: row.unit_id,
        mode,
    }))
}

impl Step<PersistWorkPosition, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &PersistWorkPosition,
    ) -> AppResult<WorkPosition> {
        let input = &operation.position;

        validate_id(&input.comic_id)?;

        sqlx::query!("SELECT id FROM comic WHERE id = ?", input.comic_id)
            .fetch_one(&mut *context.0)
            .await?;

        if let Some(page_id) = &input.page_id {
            validate_id(page_id)?;

            sqlx::query!(
                "SELECT id FROM page WHERE id = ? AND comic_id = ?",
                page_id,
                input.comic_id
            )
            .fetch_one(&mut *context.0)
            .await?;
        }

        if let Some(unit_id) = &input.unit_id {
            validate_id(unit_id)?;

            let page_id = input.page_id.as_ref().ok_or(AppError::InvalidInput)?;

            sqlx::query!(
                "SELECT id FROM unit WHERE id = ? AND page_id = ?",
                unit_id,
                page_id
            )
            .fetch_one(&mut *context.0)
            .await?;
        }

        let now = context.1;

        let mode = input.mode.as_str();

        sqlx::query!("INSERT INTO work_position (comic_id, last_opened_at, page_id, unit_id, mode) VALUES (?, ?, ?, ?, ?) ON CONFLICT(comic_id) DO UPDATE SET last_opened_at = excluded.last_opened_at, page_id = excluded.page_id, unit_id = excluded.unit_id, mode = excluded.mode", input.comic_id, now, input.page_id, input.unit_id, mode)
            .execute(&mut *context.0).await?;

        Ok(WorkPosition {
            last_opened_at: now,
            ..input.clone()
        })
    }
}
