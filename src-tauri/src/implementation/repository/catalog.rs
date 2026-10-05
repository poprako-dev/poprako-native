use poprako_orchestra::{Run, Step};
use sqlx::SqliteConnection;

use crate::data::comic::{ComicDetail, ComicInfo, PageInfo};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::read;
use crate::implementation::repository::work_position::read_position;
use crate::model::comic::Comic;
use crate::part::repository::operation::{
    DeleteComic, ListComicInfos, ReadComicDetail, UpdateComicMetadata,
};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn read_comic(connection: &mut SqliteConnection, comic_id: &str) -> AppResult<Comic> {
    validate_id(comic_id)?;

    let row = sqlx::query_as!(
        Comic,
        "SELECT id, title, subtitle, author, created_at, updated_at FROM comic WHERE id = ?",
        comic_id
    )
    .fetch_one(connection)
    .await?;

    Ok(row)
}

fn count(value: i64) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| AppError::RecoveryRequired)
}

impl Run<ListComicInfos> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ListComicInfos) -> AppResult<Vec<ComicInfo>> {
        if operation.limit == 0 || operation.limit > 100 {
            return Err(AppError::InvalidInput);
        }

        let limit = i64::from(operation.limit);

        let offset = i64::from(operation.offset);

        let rows = sqlx::query!(
            "SELECT c.id, c.title, c.subtitle, c.author, c.created_at, c.updated_at,
            (SELECT COUNT(*) FROM page p WHERE p.comic_id = c.id) AS \"page_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.comic_id = c.id) AS \"unit_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.comic_id = c.id AND u.translated_text != '') AS \"translated_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.comic_id = c.id AND u.is_proofread = 1) AS \"proofread_count!: i64\",
            (SELECT id FROM page WHERE comic_id = c.id ORDER BY position LIMIT 1) AS \"cover_page_id?: String\",
            w.last_opened_at AS \"last_opened_at?: i64\"
            FROM comic c LEFT JOIN work_position w ON w.comic_id = c.id
            ORDER BY w.last_opened_at DESC, c.updated_at DESC, c.id ASC LIMIT ? OFFSET ?",
            limit, offset
        ).fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|row| {
                Ok(ComicInfo {
                    comic: Comic {
                        id: row.id,
                        title: row.title,
                        subtitle: row.subtitle,
                        author: row.author,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    },
                    page_count: count(row.page_count)?,
                    unit_count: count(row.unit_count)?,
                    translated_count: count(row.translated_count)?,
                    proofread_count: count(row.proofread_count)?,
                    cover_page_id: row.cover_page_id,
                    last_opened_at: row.last_opened_at,
                })
            })
            .collect()
    }
}

impl Run<ReadComicDetail> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadComicDetail) -> AppResult<ComicDetail> {
        let mut transaction = self.pool.begin().await?;

        let comic = read_comic(&mut transaction, &operation.comic_id).await?;

        let rows = sqlx::query!(
            "SELECT p.id, COUNT(u.id) AS \"unit_count!: i64\", COALESCE(SUM(u.translated_text != ''), 0) AS \"translated_count!: i64\", COALESCE(SUM(u.is_proofread), 0) AS \"proofread_count!: i64\", COALESCE(SUM(u.is_flagged), 0) AS \"flagged_count!: i64\", COALESCE(SUM(u.translated_text != '' AND u.proofread_text != '' AND u.translated_text != u.proofread_text), 0) AS \"edited_count!: i64\", COALESCE(SUM(u.translated_text = '' AND u.proofread_text != ''), 0) AS \"proofreader_append_count!: i64\" FROM page p LEFT JOIN unit u ON u.page_id = p.id WHERE p.comic_id = ? GROUP BY p.id ORDER BY p.position",
            operation.comic_id
        ).fetch_all(&mut *transaction).await?;

        let mut pages = Vec::with_capacity(rows.len());

        for row in rows {
            pages.push(PageInfo {
                page: read::page(&mut transaction, &operation.comic_id, &row.id).await?,
                unit_count: count(row.unit_count)?,
                translated_count: count(row.translated_count)?,
                proofread_count: count(row.proofread_count)?,
                flagged_count: count(row.flagged_count)?,
                edited_count: count(row.edited_count)?,
                proofreader_append_count: count(row.proofreader_append_count)?,
            });
        }

        let work_position = read_position(&mut transaction, &operation.comic_id).await?;

        transaction.commit().await?;

        Ok(ComicDetail {
            comic,
            pages,
            work_position,
        })
    }
}

impl Step<UpdateComicMetadata, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &UpdateComicMetadata,
    ) -> AppResult<Comic> {
        let before = read_comic(&mut context.0, &operation.comic_id).await?;

        let title = operation.metadata.title.trim().to_owned();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let subtitle = operation.metadata.subtitle.trim().to_owned();

        let author = operation.metadata.author.trim().to_owned();

        if before.title == title && before.subtitle == subtitle && before.author == author {
            return Ok(before);
        }

        let now = context.1;

        sqlx::query!(
            "UPDATE comic SET title = ?, subtitle = ?, author = ?, updated_at = ? WHERE id = ?",
            title,
            subtitle,
            author,
            now,
            operation.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        Ok(Comic {
            title,
            subtitle,
            author,
            updated_at: now,
            ..before
        })
    }
}

impl Step<DeleteComic, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &DeleteComic) -> AppResult<()> {
        validate_id(&operation.comic_id)?;

        let result = sqlx::query!("DELETE FROM comic WHERE id = ?", operation.comic_id)
            .execute(&mut *context.0)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}
