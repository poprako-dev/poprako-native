use poprako_orchestra::Step;
use sqlx::SqliteConnection;

use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::catalog::read_chapter;
use crate::implementation::repository::chapter_order::{read_order, validate_baseline};
use crate::model::chapter::Chapter;
use crate::part::repository::operation::CreateChapter;
use crate::result::{AppError, AppResult};
use crate::value::validation::{position, validate_id};

/// # Errors
/// Propagates a real content modification to its chapter and owning comic.
pub async fn touch(connection: &mut SqliteConnection, chapter_id: &str, now: i64) -> AppResult<()> {
    sqlx::query!(
        "UPDATE chapter SET updated_at = ? WHERE id = ?",
        now,
        chapter_id
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query!(
        "UPDATE comic SET updated_at = ? WHERE id = (SELECT comic_id FROM chapter WHERE id = ?)",
        now,
        chapter_id
    )
    .execute(connection)
    .await?;

    Ok(())
}

impl Step<CreateChapter, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &CreateChapter,
    ) -> AppResult<Chapter> {
        validate_id(&operation.id)?;

        let order = read_order(&mut context.0, &operation.comic_id).await?;

        validate_baseline(&order, &operation.baseline)?;

        let title = operation.metadata.title.trim();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let index = i64::from(position(order.len())?);

        let now = context.1;

        sqlx::query!("INSERT INTO chapter (id, comic_id, title, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)", operation.id, operation.comic_id, title, index, now, now)
            .execute(&mut *context.0).await?;

        touch(&mut context.0, &operation.id, now).await?;

        read_chapter(&mut context.0, &operation.id).await
    }
}
