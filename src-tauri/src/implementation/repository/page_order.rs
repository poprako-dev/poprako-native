use std::collections::{HashMap, HashSet};

use poprako_orchestra::Step;
use sqlx::SqliteConnection;

use crate::data::page::PageBaseline;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::page::{matches_baseline, read_pages};
use crate::model::page::Page;
use crate::part::repository::page::{DeletePages, PersistPageOrder};
use crate::result::{AppError, AppResult};
use crate::value::validation::position;

fn validate_baseline(pages: &[Page], baseline: &[PageBaseline]) -> AppResult<()> {
    if pages.len() != baseline.len()
        || !pages
            .iter()
            .zip(baseline)
            .all(|(page, baseline)| matches_baseline(page, baseline))
    {
        return Err(AppError::Conflict);
    }

    Ok(())
}

async fn reorder(
    connection: &mut SqliteConnection,
    comic_id: &str,
    old: &[Page],
    ids: &[String],
    now: i64,
) -> AppResult<()> {
    let previous: HashMap<_, _> = old
        .iter()
        .map(|page| (page.id.as_str(), page.index))
        .collect();

    let offset = i64::try_from(old.len())
        .map_err(|_| AppError::InvalidInput)?
        .checked_add(1)
        .ok_or(AppError::InvalidInput)?;

    let mut changes = Vec::new();

    for (index, id) in ids.iter().enumerate() {
        let index = position(index)?;

        let old_index = previous.get(id.as_str()).ok_or(AppError::InvalidInput)?;

        if *old_index == index {
            continue;
        }

        let temporary = offset
            .checked_add(i64::from(*old_index))
            .ok_or(AppError::InvalidInput)?;

        sqlx::query!(
            "UPDATE page SET position = ? WHERE id = ? AND comic_id = ?",
            temporary,
            id,
            comic_id
        )
        .execute(&mut *connection)
        .await?;

        changes.push((id, i64::from(index)));
    }

    for (id, index) in changes {
        sqlx::query!(
            "UPDATE page SET position = ?, updated_at = ? WHERE id = ? AND comic_id = ?",
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

impl Step<PersistPageOrder, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &PersistPageOrder,
    ) -> AppResult<Vec<Page>> {
        let input = &operation.input;

        let before = read_pages(&mut context.0, &input.comic_id).await?;

        validate_baseline(&before, &input.baseline)?;

        let expected: HashSet<_> = before.iter().map(|page| &page.id).collect();

        let actual: HashSet<_> = input.page_ids.iter().collect();

        if expected != actual || input.page_ids.len() != actual.len() {
            return Err(AppError::InvalidInput);
        }

        if before.iter().map(|page| &page.id).eq(input.page_ids.iter()) {
            return Ok(before);
        }

        let now = context.1;

        reorder(
            &mut context.0,
            &input.comic_id,
            &before,
            &input.page_ids,
            now,
        )
        .await?;

        sqlx::query!(
            "UPDATE comic SET updated_at = ? WHERE id = ?",
            now,
            input.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read_pages(&mut context.0, &input.comic_id).await
    }
}

impl Step<DeletePages, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &DeletePages,
    ) -> AppResult<Vec<Page>> {
        let input = &operation.input;

        let before = read_pages(&mut context.0, &input.comic_id).await?;

        validate_baseline(&before, &input.baseline)?;

        let removed: HashSet<_> = input.page_ids.iter().collect();

        if removed.len() != input.page_ids.len()
            || removed
                .iter()
                .any(|id| !before.iter().any(|page| &page.id == *id))
        {
            return Err(AppError::InvalidInput);
        }

        if removed.is_empty() {
            return Ok(before);
        }

        let survivors: Vec<_> = before
            .iter()
            .filter(|page| !removed.contains(&page.id))
            .map(|page| page.id.clone())
            .collect();

        let now = context.1;

        for id in &input.page_ids {
            sqlx::query!(
                "UPDATE work_position SET unit_id = NULL, page_id = NULL WHERE page_id = ?",
                id
            )
            .execute(&mut *context.0)
            .await?;

            sqlx::query!(
                "DELETE FROM page WHERE id = ? AND comic_id = ?",
                id,
                input.comic_id
            )
            .execute(&mut *context.0)
            .await?;
        }

        reorder(&mut context.0, &input.comic_id, &before, &survivors, now).await?;

        let first = survivors.first();

        sqlx::query!("UPDATE work_position SET page_id = ?, unit_id = NULL WHERE comic_id = ? AND page_id IS NULL", first, input.comic_id)
            .execute(&mut *context.0).await?;

        sqlx::query!(
            "UPDATE comic SET updated_at = ? WHERE id = ?",
            now,
            input.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read_pages(&mut context.0, &input.comic_id).await
    }
}
