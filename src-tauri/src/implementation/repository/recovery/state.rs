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
) -> AppResult<Vec<Unit>> {
    let page_ids =
        serde_json::to_string(&scope.unit_page_ids).map_err(|_| AppError::RecoveryRequired)?;

    let unit_ids =
        serde_json::to_string(&scope.unit_ids).map_err(|_| AppError::RecoveryRequired)?;

    let rows = sqlx::query!(
        "SELECT id, page_id, position, x_coord, y_coord, is_bubble, is_flagged, translated_text, proofread_text, is_proofread, created_at, updated_at FROM unit WHERE (? AND page_id IN (SELECT id FROM page WHERE comic_id = ?)) OR page_id IN (SELECT value FROM json_each(?)) OR id IN (SELECT value FROM json_each(?)) ORDER BY id",
        scope.comic_units, scope.comic_id, page_ids, unit_ids
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
    let comic = match scope.comic_row {
        true => sqlx::query_as!(
            Comic,
            "SELECT id, title, subtitle, author, created_at, updated_at FROM comic WHERE id = ?",
            scope.comic_id
        )
        .fetch_optional(&mut *connection)
        .await?,
        false => None,
    };

    let page_ids =
        serde_json::to_string(&scope.page_ids).map_err(|_| AppError::RecoveryRequired)?;

    let rows = sqlx::query!(
        "SELECT id, comic_id FROM page WHERE (? AND comic_id = ?) OR id IN (SELECT value FROM json_each(?)) ORDER BY id",
        scope.comic_pages, scope.comic_id, page_ids
    ).fetch_all(&mut *connection).await?;

    let mut pages = Vec::with_capacity(rows.len());

    for row in rows {
        pages.push(read::page(connection, &row.comic_id, &row.id).await?);
    }

    let units = capture_units(connection, scope).await?;

    let work_position = match (scope.work_position, &scope.comic_id) {
        (true, Some(comic_id)) => read_position(connection, comic_id).await?,
        _ => None,
    };

    let preference_payload = match scope.preference {
        true => {
            sqlx::query_scalar!("SELECT payload FROM application_preference WHERE singleton = 1")
                .fetch_optional(&mut *connection)
                .await?
        }
        false => None,
    };

    Ok(MutationState {
        comic,
        pages,
        units,
        work_position,
        preference_payload,
    })
}
