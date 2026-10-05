use poprako_orchestra::Step;
use sqlx::SqliteConnection;

use crate::data::page::PageBaseline;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::catalog::read_comic;
use crate::implementation::repository::read;
use crate::model::page::Page;
use crate::part::repository::page::{AppendPages, ReplacePageImage};
use crate::result::{AppError, AppResult};
use crate::value::image::Image;
use crate::value::validation::{MAX_SAFE_INTEGER, position, validate_id};

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub fn validate_image(image: &Image) -> AppResult<()> {
    if image.width == 0
        || image.height == 0
        || image.width > 16384
        || image.height > 16384
        || u64::from(image.width) * u64::from(image.height) > 32_000_000
        || image.reference.is_empty()
        || image.reference.starts_with('/')
        || image.reference.contains('\\')
        || image.reference.contains(':')
        || image
            .reference
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(AppError::InvalidInput);
    }

    Ok(())
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn read_pages(connection: &mut SqliteConnection, comic_id: &str) -> AppResult<Vec<Page>> {
    read_comic(connection, comic_id).await?;

    let rows = sqlx::query!(
        "SELECT id FROM page WHERE comic_id = ? ORDER BY position",
        comic_id
    )
    .fetch_all(&mut *connection)
    .await?;

    let mut pages = Vec::with_capacity(rows.len());

    for row in rows {
        pages.push(read::page(connection, comic_id, &row.id).await?);
    }

    Ok(pages)
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
#[must_use]
pub fn matches_baseline(page: &Page, baseline: &PageBaseline) -> bool {
    page.id == baseline.id
        && page.unit_revision == baseline.unit_revision
        && page.image.reference == baseline.image_reference
}

impl Step<AppendPages, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &AppendPages,
    ) -> AppResult<Vec<Page>> {
        let existing = read_pages(&mut context.0, &operation.comic_id).await?;

        if existing.len() != operation.baseline.len()
            || !existing
                .iter()
                .zip(&operation.baseline)
                .all(|(page, baseline)| matches_baseline(page, baseline))
        {
            return Err(AppError::Conflict);
        }

        if operation.pages.is_empty() {
            return Ok(existing);
        }

        let total = existing
            .len()
            .checked_add(operation.pages.len())
            .ok_or(AppError::InvalidInput)?;

        position(total)?;

        let now = context.1;

        for (index, prepared) in operation.pages.iter().enumerate() {
            let image = &prepared.image;

            validate_id(&prepared.id)?;

            validate_image(image)?;

            let id = &prepared.id;

            let ordinal = i64::from(position(existing.len() + index)?);

            let format = image.format.as_str();

            let width = i64::from(image.width);

            let height = i64::from(image.height);

            sqlx::query!("INSERT INTO page (id, comic_id, position, unit_revision, image_reference, image_original_name, image_format, image_width, image_height, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?)",
                id, operation.comic_id, ordinal, image.reference, image.original_name, format, width, height, now, now)
                .execute(&mut *context.0).await?;
        }

        sqlx::query!(
            "UPDATE comic SET updated_at = ? WHERE id = ?",
            now,
            operation.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read_pages(&mut context.0, &operation.comic_id).await
    }
}

impl Step<ReplacePageImage, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ReplacePageImage,
    ) -> AppResult<Page> {
        validate_image(&operation.image)?;

        let page = read::page(&mut context.0, &operation.comic_id, &operation.baseline.id).await?;

        if !matches_baseline(&page, &operation.baseline) {
            return Err(AppError::Conflict);
        }

        let mut revision = page.unit_revision;

        if operation.clear_units {
            sqlx::query!(
                "UPDATE work_position SET unit_id = NULL WHERE page_id = ?",
                page.id
            )
            .execute(&mut *context.0)
            .await?;

            let result = sqlx::query!("DELETE FROM unit WHERE page_id = ?", page.id)
                .execute(&mut *context.0)
                .await?;

            if result.rows_affected() > 0 {
                revision = revision
                    .checked_add(1)
                    .filter(|value| *value <= MAX_SAFE_INTEGER)
                    .ok_or(AppError::RecoveryRequired)?;
            }
        }

        if page.image == operation.image && revision == page.unit_revision {
            return Ok(page);
        }

        let now = context.1;

        let image = &operation.image;

        let format = image.format.as_str();

        let width = i64::from(image.width);

        let height = i64::from(image.height);

        sqlx::query!("UPDATE page SET image_reference = ?, image_original_name = ?, image_format = ?, image_width = ?, image_height = ?, unit_revision = ?, updated_at = ? WHERE id = ?", image.reference, image.original_name, format, width, height, revision, now, page.id)
            .execute(&mut *context.0).await?;

        sqlx::query!(
            "UPDATE comic SET updated_at = ? WHERE id = ?",
            now,
            operation.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read::page(&mut context.0, &operation.comic_id, &page.id).await
    }
}

impl poprako_orchestra::Run<crate::part::repository::page::ReadPage> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &crate::part::repository::page::ReadPage) -> AppResult<Page> {
        validate_id(&operation.comic_id)?;

        validate_id(&operation.page_id)?;

        let mut connection = self.pool.acquire().await?;

        read::page(&mut connection, &operation.comic_id, &operation.page_id).await
    }
}
