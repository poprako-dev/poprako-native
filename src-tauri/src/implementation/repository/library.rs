use poprako_orchestra::Run;
use sqlx::SqliteConnection;

use crate::data::comic::ComicInfo;
use crate::implementation::repository::SqliteRepository;
use crate::model::comic::Comic;
use crate::part::repository::comic::ListComicInfos;
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

/// # Errors
/// Rejects invalid identities and unavailable parents.
pub async fn read_comic(connection: &mut SqliteConnection, id: &str) -> AppResult<Comic> {
    validate_id(id)?;

    Ok(sqlx::query_as!(
        Comic,
        "SELECT id, title, author, created_at, updated_at FROM comic WHERE id = ?",
        id
    )
    .fetch_one(connection)
    .await?)
}

/// # Errors
/// Updates only the affected parent's content time in the caller's transaction.
pub async fn touch_comic(connection: &mut SqliteConnection, id: &str, now: i64) -> AppResult<()> {
    sqlx::query!("UPDATE comic SET updated_at = ? WHERE id = ?", now, id)
        .execute(connection)
        .await?;

    Ok(())
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
            r#"SELECT c.id, c.title, c.author, c.created_at, c.updated_at,
            (SELECT COUNT(*) FROM chapter h WHERE h.comic_id = c.id) AS "chapter_count!: i64",
            (SELECT COUNT(*) FROM page p JOIN chapter h ON h.id = p.chapter_id WHERE h.comic_id = c.id) AS "page_count!: i64",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id JOIN chapter h ON h.id = p.chapter_id WHERE h.comic_id = c.id) AS "unit_count!: i64",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id JOIN chapter h ON h.id = p.chapter_id WHERE h.comic_id = c.id AND u.translated_text != '') AS "translated_count!: i64",
            (SELECT COUNT(*) FROM unit u JOIN page p ON p.id = u.page_id JOIN chapter h ON h.id = p.chapter_id WHERE h.comic_id = c.id AND u.is_proofread = 1) AS "proofread_count!: i64",
            (SELECT p.id FROM chapter h JOIN page p ON p.chapter_id = h.id WHERE h.comic_id = c.id ORDER BY h.position, p.position LIMIT 1) AS "cover_page_id?: String",
            (SELECT h.id FROM chapter h JOIN page p ON p.chapter_id = h.id WHERE h.comic_id = c.id ORDER BY h.position, p.position LIMIT 1) AS "cover_chapter_id?: String",
            (SELECT h.id FROM chapter h LEFT JOIN work_position w ON w.chapter_id = h.id WHERE h.comic_id = c.id ORDER BY w.last_opened_at DESC, h.position LIMIT 1) AS "latest_chapter_id?: String",
            (SELECT MAX(w.last_opened_at) FROM chapter h JOIN work_position w ON w.chapter_id = h.id WHERE h.comic_id = c.id) AS "last_opened_at?: i64"
            FROM comic c ORDER BY "last_opened_at?: i64" DESC, c.updated_at DESC, c.id LIMIT ? OFFSET ?"#, limit, offset
        ).fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|row| {
                Ok(ComicInfo {
                    comic: Comic {
                        id: row.id,
                        title: row.title,
                        author: row.author,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    },
                    chapter_count: count(row.chapter_count)?,
                    page_count: count(row.page_count)?,
                    unit_count: count(row.unit_count)?,
                    translated_count: count(row.translated_count)?,
                    proofread_count: count(row.proofread_count)?,
                    cover_page_id: row.cover_page_id,
                    cover_chapter_id: row.cover_chapter_id,
                    latest_chapter_id: row.latest_chapter_id,
                    last_opened_at: row.last_opened_at,
                })
            })
            .collect()
    }
}

impl Run<crate::part::repository::comic::ReadComic> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &crate::part::repository::comic::ReadComic) -> AppResult<Comic> {
        read_comic(&mut *self.pool.acquire().await?, &operation.comic_id).await
    }
}
