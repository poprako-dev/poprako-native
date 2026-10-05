use sqlx::SqliteConnection;

use crate::data::editor::PageEditor;
use crate::model::page::Page;
use crate::model::unit::Unit;
use crate::result::{AppError, AppResult};
use crate::value::image::{Image, ImageFormat};
use crate::value::validation::MAX_SAFE_INTEGER;

fn checked_time(value: i64) -> AppResult<i64> {
    if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value) {
        return Err(AppError::RecoveryRequired);
    }

    Ok(value)
}

fn checked_position(value: i64) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| AppError::RecoveryRequired)
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn page(
    connection: &mut SqliteConnection,
    comic_id: &str,
    page_id: &str,
) -> AppResult<Page> {
    let row = sqlx::query!(
        "SELECT id, comic_id, position, unit_revision, image_reference, image_original_name, image_format, image_width, image_height, created_at, updated_at FROM page WHERE id = ? AND comic_id = ?",
        page_id, comic_id
    ).fetch_one(&mut *connection).await?;

    let format = match row.image_format.as_str() {
        "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::Webp,
        "bmp" => ImageFormat::Bmp,
        _ => return Err(AppError::RecoveryRequired),
    };

    Ok(Page {
        id: row.id,
        comic_id: row.comic_id,
        index: checked_position(row.position)?,
        unit_revision: checked_time(row.unit_revision)?,
        image: Image {
            reference: row.image_reference,
            original_name: row.image_original_name,
            format,
            width: checked_position(row.image_width)?,
            height: checked_position(row.image_height)?,
        },
        created_at: checked_time(row.created_at)?,
        updated_at: checked_time(row.updated_at)?,
    })
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn units(connection: &mut SqliteConnection, page_id: &str) -> AppResult<Vec<Unit>> {
    let rows = sqlx::query!(
        "SELECT id, page_id, position, x_coord, y_coord, is_bubble, is_flagged, translated_text, proofread_text, is_proofread, created_at, updated_at FROM unit WHERE page_id = ? ORDER BY position",
        page_id
    ).fetch_all(&mut *connection).await?;

    rows.into_iter()
        .map(|row| {
            Ok(Unit {
                id: row.id,
                page_id: row.page_id,
                index: checked_position(row.position)?,
                x_coord: row.x_coord,
                y_coord: row.y_coord,
                is_bubble: row.is_bubble != 0,
                is_flagged: row.is_flagged != 0,
                translated_text: row.translated_text,
                proofread_text: row.proofread_text,
                is_proofread: row.is_proofread != 0,
                created_at: checked_time(row.created_at)?,
                updated_at: checked_time(row.updated_at)?,
            })
        })
        .collect()
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn editor(
    connection: &mut SqliteConnection,
    comic_id: &str,
    page_id: &str,
) -> AppResult<PageEditor> {
    let page = page(connection, comic_id, page_id).await?;

    let units = units(connection, page_id).await?;

    Ok(PageEditor { page, units })
}
