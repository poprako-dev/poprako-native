use std::collections::HashSet;

use poprako_orchestra::Step;
use sqlx::SqliteConnection;

use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::catalog::read_chapter;
use crate::implementation::repository::library::{read_comic, touch_comic};
use crate::model::chapter::Chapter;
use crate::part::repository::comic::{RelocateChapter, ReorderChapters};
use crate::result::{AppError, AppResult};
use crate::value::validation::position;

/// # Errors
/// Missing parents are distinct from existing parents with no chapters.
pub async fn read_order(
    connection: &mut SqliteConnection,
    comic_id: &str,
) -> AppResult<Vec<String>> {
    read_comic(connection, comic_id).await?;

    Ok(sqlx::query_scalar!(
        "SELECT id FROM chapter WHERE comic_id = ? ORDER BY position",
        comic_id
    )
    .fetch_all(connection)
    .await?)
}

/// # Errors
/// Rejects stale or partial chapter lists.
pub fn validate_baseline(actual: &[String], baseline: &[String]) -> AppResult<()> {
    if actual != baseline {
        return Err(AppError::Conflict);
    }

    Ok(())
}

/// # Errors
/// Uses a disjoint nonnegative interval to preserve immediate uniqueness constraints.
pub async fn reorder(
    connection: &mut SqliteConnection,
    comic_id: &str,
    before: &[String],
    after: &[String],
    now: i64,
) -> AppResult<()> {
    let offset = i64::try_from(before.len())
        .map_err(|_| AppError::InvalidInput)?
        .checked_add(1)
        .ok_or(AppError::InvalidInput)?;

    let mut changes = Vec::new();

    for (index, id) in after.iter().enumerate() {
        let old = before
            .iter()
            .position(|value| value == id)
            .ok_or(AppError::InvalidInput)?;

        if old == index {
            continue;
        }

        let temporary = offset
            .checked_add(i64::from(position(old)?))
            .ok_or(AppError::InvalidInput)?;

        sqlx::query!(
            "UPDATE chapter SET position = ? WHERE id = ? AND comic_id = ?",
            temporary,
            id,
            comic_id
        )
        .execute(&mut *connection)
        .await?;

        changes.push((id, i64::from(position(index)?)));
    }

    for (id, index) in changes {
        sqlx::query!(
            "UPDATE chapter SET position = ?, updated_at = ? WHERE id = ? AND comic_id = ?",
            index,
            now,
            id,
            comic_id
        )
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

impl Step<ReorderChapters, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ReorderChapters,
    ) -> AppResult<()> {
        let input = &operation.input;

        let before = read_order(&mut context.0, &input.comic_id).await?;

        validate_baseline(&before, &input.baseline)?;

        let expected: HashSet<_> = before.iter().collect();

        let actual: HashSet<_> = input.ordered_ids.iter().collect();

        if expected != actual || actual.len() != input.ordered_ids.len() {
            return Err(AppError::InvalidInput);
        }

        if before == input.ordered_ids {
            return Ok(());
        }

        reorder(
            &mut context.0,
            &input.comic_id,
            &before,
            &input.ordered_ids,
            context.1,
        )
        .await?;

        touch_comic(&mut context.0, &input.comic_id, context.1).await
    }
}

impl Step<RelocateChapter, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &RelocateChapter,
    ) -> AppResult<Chapter> {
        let input = &operation.input;

        if input.source_comic_id == input.target_comic_id {
            return Err(AppError::InvalidInput);
        }

        let source = read_order(&mut context.0, &input.source_comic_id).await?;

        let target = read_order(&mut context.0, &input.target_comic_id).await?;

        validate_baseline(&source, &input.source_baseline)?;

        validate_baseline(&target, &input.target_baseline)?;

        if !source.contains(&input.chapter_id) {
            return Err(AppError::Conflict);
        }

        let index = i64::from(position(target.len())?);

        let now = context.1;

        sqlx::query!(
            "UPDATE chapter SET comic_id = ?, position = ?, updated_at = ? WHERE id = ?",
            input.target_comic_id,
            index,
            now,
            input.chapter_id
        )
        .execute(&mut *context.0)
        .await?;

        let after: Vec<_> = source
            .iter()
            .filter(|id| **id != input.chapter_id)
            .cloned()
            .collect();

        reorder(&mut context.0, &input.source_comic_id, &source, &after, now).await?;

        touch_comic(&mut context.0, &input.source_comic_id, now).await?;

        touch_comic(&mut context.0, &input.target_comic_id, now).await?;

        read_chapter(&mut context.0, &input.chapter_id).await
    }
}
