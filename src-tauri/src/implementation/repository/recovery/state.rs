use sqlx::SqliteConnection;

use crate::data::recovery::{MutationState, RecoveryScope};
use crate::implementation::repository::read;
use crate::implementation::repository::work_position::read_position;
use crate::model::comic::Comic;
use crate::model::unit::Unit;
use crate::result::{AppError, AppResult};
use crate::value::validation::MAX_SAFE_INTEGER;

fn checked_integer(value: i64) -> AppResult<i64> {
    if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value) {
        return Err(AppError::RecoveryRequired);
    }

    Ok(value)
}

async fn capture_units(
    connection: &mut SqliteConnection,
    scope: &RecoveryScope,
    chapter_ids: &str,
) -> AppResult<Vec<Unit>> {
    let page_ids =
        serde_json::to_string(&scope.unit_page_ids).map_err(|_| AppError::RecoveryRequired)?;

    let unit_ids =
        serde_json::to_string(&scope.unit_ids).map_err(|_| AppError::RecoveryRequired)?;

    let rows = sqlx::query!(
        "SELECT id, page_id, position, x_coord, y_coord, is_bubble, is_flagged, translated_text, proofread_text, is_proofread, created_at, updated_at FROM unit WHERE (? AND page_id IN (SELECT id FROM page WHERE chapter_id IN (SELECT value FROM json_each(?)))) OR page_id IN (SELECT value FROM json_each(?)) OR id IN (SELECT value FROM json_each(?)) ORDER BY id",
        scope.chapter_units, chapter_ids, page_ids, unit_ids
    ).fetch_all(connection).await?;

    rows.into_iter()
        .map(|row| {
            Ok(Unit {
                id: row.id,
                page_id: row.page_id,
                index: u32::try_from(row.position).map_err(|_| AppError::RecoveryRequired)?,
                x_coord: row.x_coord,
                y_coord: row.y_coord,
                is_bubble: row.is_bubble != 0,
                is_flagged: row.is_flagged != 0,
                translated_text: row.translated_text,
                proofread_text: row.proofread_text,
                is_proofread: row.is_proofread != 0,
                created_at: checked_integer(row.created_at)?,
                updated_at: checked_integer(row.updated_at)?,
            })
        })
        .collect()
}

/// # Errors
/// Reports unavailable or unrepresentable rows; no partial snapshot is returned.
pub async fn capture(
    connection: &mut SqliteConnection,
    scope: &RecoveryScope,
) -> AppResult<MutationState> {
    let parent_ids =
        serde_json::to_string(&scope.comic_ids).map_err(|_| AppError::RecoveryRequired)?;

    let comics = sqlx::query_as!(Comic,
        "SELECT id, title, author, created_at, updated_at FROM comic WHERE id IN (SELECT value FROM json_each(?)) OR id = (SELECT comic_id FROM chapter WHERE id = ?) ORDER BY id",
        parent_ids, scope.chapter_id).fetch_all(&mut *connection).await?;

    let ids = sqlx::query_scalar!("SELECT id FROM chapter WHERE comic_id IN (SELECT value FROM json_each(?)) OR (? AND id = ?) ORDER BY id", parent_ids, scope.chapter_row, scope.chapter_id).fetch_all(&mut *connection).await?;

    let mut chapters = Vec::new();

    for id in &ids {
        chapters
            .push(crate::implementation::repository::catalog::read_chapter(connection, id).await?);
    }

    let chapter_ids = serde_json::to_string(&ids).map_err(|_| AppError::RecoveryRequired)?;

    let page_ids =
        serde_json::to_string(&scope.page_ids).map_err(|_| AppError::RecoveryRequired)?;

    let rows = sqlx::query!(
        "SELECT id, chapter_id FROM page WHERE (? AND chapter_id IN (SELECT value FROM json_each(?))) OR id IN (SELECT value FROM json_each(?)) ORDER BY id",
        scope.chapter_pages, chapter_ids, page_ids
    ).fetch_all(&mut *connection).await?;

    let mut pages = Vec::with_capacity(rows.len());

    for row in rows {
        pages.push(read::page(connection, &row.chapter_id, &row.id).await?);
    }

    let units = capture_units(connection, scope, &chapter_ids).await?;

    let mut work_positions = Vec::new();

    if scope.work_position {
        for id in &ids {
            if let Some(value) = read_position(connection, id).await? {
                work_positions.push(value);
            }
        }
    }

    let preference_payload = match scope.preference {
        true => {
            sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
                .fetch_optional(&mut *connection)
                .await?
        }
        false => None,
    };

    let (termbases, terms) =
        crate::implementation::repository::terminology::recovery::capture(connection, scope)
            .await?;

    Ok(MutationState {
        termbases,
        terms,
        comics,
        chapters,
        pages,
        units,
        work_positions,
        preference_payload,
    })
}
