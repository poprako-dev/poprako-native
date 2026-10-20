use poprako_orchestra::{Run, Step};
use sqlx::SqliteConnection;

use crate::data::chapter::{ChapterDetail, ChapterInfo, PageInfo};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::read;
use crate::implementation::repository::work_position::read_position;
use crate::model::chapter::Chapter;
use crate::part::repository::operation::{
    DeleteChapter, ListChapterInfos, ReadChapterDetail, UpdateChapterMetadata,
};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn read_chapter(
    connection: &mut SqliteConnection,
    chapter_id: &str,
) -> AppResult<Chapter> {
    validate_id(chapter_id)?;

    let row = sqlx::query!(
        "SELECT id, comic_id, title, position, created_at, updated_at FROM chapter WHERE id = ?",
        chapter_id
    )
    .fetch_one(connection)
    .await?;

    Ok(Chapter {
        id: row.id,
        comic_id: row.comic_id,
        title: row.title,
        index: u32::try_from(row.position).map_err(|_| AppError::RecoveryRequired)?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn count(value: i64) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| AppError::RecoveryRequired)
}

impl Run<ListChapterInfos> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ListChapterInfos) -> AppResult<Vec<ChapterInfo>> {
        validate_id(&operation.comic_id)?;

        crate::implementation::repository::library::read_comic(
            &mut *self.pool.acquire().await?,
            &operation.comic_id,
        )
        .await?;

        let rows = sqlx::query!(
            "SELECT c.id, c.comic_id, c.title, c.position, c.created_at, c.updated_at,
            (SELECT COUNT(*) FROM page p WHERE p.chapter_id = c.id) AS \"page_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.chapter_id = c.id) AS \"unit_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.chapter_id = c.id AND u.translated_text != '') AS \"translated_count!: i64\",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id WHERE p.chapter_id = c.id AND u.is_proofread = 1) AS \"proofread_count!: i64\",
            (SELECT id FROM page WHERE chapter_id = c.id ORDER BY position LIMIT 1) AS \"cover_page_id?: String\",
            w.last_opened_at AS \"last_opened_at?: i64\"
            FROM chapter c LEFT JOIN work_position w ON w.chapter_id = c.id
            WHERE c.comic_id = ? ORDER BY c.position",
            operation.comic_id
        ).fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|row| {
                Ok(ChapterInfo {
                    chapter: Chapter {
                        id: row.id,
                        title: row.title,
                        comic_id: row.comic_id,
                        index: count(row.position)?,
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

impl Run<ReadChapterDetail> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadChapterDetail) -> AppResult<ChapterDetail> {
        let mut transaction = self.pool.begin().await?;

        let chapter = read_chapter(&mut transaction, &operation.chapter_id).await?;

        let rows = sqlx::query!(
            "SELECT p.id, COUNT(u.id) AS \"unit_count!: i64\", COALESCE(SUM(u.translated_text != ''), 0) AS \"translated_count!: i64\", COALESCE(SUM(u.is_proofread), 0) AS \"proofread_count!: i64\", COALESCE(SUM(u.is_flagged), 0) AS \"flagged_count!: i64\", COALESCE(SUM(u.translated_text != '' AND u.proofread_text != '' AND u.translated_text != u.proofread_text), 0) AS \"edited_count!: i64\", COALESCE(SUM(u.translated_text = '' AND u.proofread_text != ''), 0) AS \"proofreader_append_count!: i64\" FROM page p LEFT JOIN unit u ON u.page_id = p.id WHERE p.chapter_id = ? GROUP BY p.id ORDER BY p.position",
            operation.chapter_id
        ).fetch_all(&mut *transaction).await?;

        let mut pages = Vec::with_capacity(rows.len());

        for row in rows {
            pages.push(PageInfo {
                page: read::page(&mut transaction, &operation.chapter_id, &row.id).await?,
                unit_count: count(row.unit_count)?,
                translated_count: count(row.translated_count)?,
                proofread_count: count(row.proofread_count)?,
                flagged_count: count(row.flagged_count)?,
                edited_count: count(row.edited_count)?,
                proofreader_append_count: count(row.proofreader_append_count)?,
            });
        }

        let work_position = read_position(&mut transaction, &operation.chapter_id).await?;

        let comic = crate::implementation::repository::library::read_comic(
            &mut transaction,
            &chapter.comic_id,
        )
        .await?;

        transaction.commit().await?;

        Ok(ChapterDetail {
            comic,
            chapter,
            pages,
            work_position,
        })
    }
}

impl Step<UpdateChapterMetadata, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &UpdateChapterMetadata,
    ) -> AppResult<Chapter> {
        let before = read_chapter(&mut context.0, &operation.chapter_id).await?;

        let title = operation.metadata.title.trim().to_owned();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        if before.title == title {
            return Ok(before);
        }

        let now = context.1;

        sqlx::query!(
            "UPDATE chapter SET title = ?, updated_at = ? WHERE id = ?",
            title,
            now,
            operation.chapter_id
        )
        .execute(&mut *context.0)
        .await?;

        crate::implementation::repository::chapter::touch(
            &mut context.0,
            &operation.chapter_id,
            now,
        )
        .await?;

        Ok(Chapter {
            title,
            updated_at: now,
            ..before
        })
    }
}

impl Step<DeleteChapter, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &DeleteChapter) -> AppResult<()> {
        let chapter = read_chapter(&mut context.0, &operation.chapter_id).await?;

        let before = crate::implementation::repository::chapter_order::read_order(
            &mut context.0,
            &chapter.comic_id,
        )
        .await?;

        sqlx::query!("DELETE FROM chapter WHERE id = ?", operation.chapter_id)
            .execute(&mut *context.0)
            .await?;

        let after: Vec<_> = before
            .iter()
            .filter(|id| **id != operation.chapter_id)
            .cloned()
            .collect();

        crate::implementation::repository::chapter_order::reorder(
            &mut context.0,
            &chapter.comic_id,
            &before,
            &after,
            context.1,
        )
        .await?;

        crate::implementation::repository::library::touch_comic(
            &mut context.0,
            &chapter.comic_id,
            context.1,
        )
        .await?;

        Ok(())
    }
}
