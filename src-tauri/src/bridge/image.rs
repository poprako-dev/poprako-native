use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use uuid::Uuid;

use crate::bridge::error::CommandResult;
use crate::harness::Harness;
use crate::implementation::resource::cache::DisplayRequest;
use crate::implementation::resource::registry::ImageGrant;
use crate::result::AppError;
use crate::usecase::page;

#[derive(Clone, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ImageKind {
    Thumbnail,
    Preview,
}

#[derive(Serialize, Type)]
pub struct ImageResource {
    pub handle: String,
    pub width: u32,
    pub height: u32,
}

async fn acquire(
    state: &Harness,
    comic_id: String,
    page_id: String,
    request: DisplayRequest,
) -> CommandResult<ImageResource> {
    let coordinator = state.database().await?;

    let _barrier = coordinator.recovery_barrier().await;

    let page = page::get_page(coordinator, &comic_id, &page_id).await?;

    let grant = ImageGrant {
        comic_id: Uuid::parse_str(&comic_id).map_err(|_| AppError::InvalidInput)?,
        page_id: Uuid::parse_str(&page_id).map_err(|_| AppError::InvalidInput)?,
        reference: page.image.reference,
        request,
    };

    let handle = state.resource_registry.acquire(grant)?;

    Ok(ImageResource {
        handle,
        width: page.image.width,
        height: page.image.height,
    })
}

/// # Errors
/// Rejects invalid page ownership or exhausted resource handles.
#[tauri::command]
#[specta::specta]
pub async fn get_image_resource(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    page_id: String,
    kind: ImageKind,
) -> CommandResult<ImageResource> {
    let request = match kind {
        ImageKind::Thumbnail => DisplayRequest::Thumbnail,
        ImageKind::Preview => DisplayRequest::Preview,
    };

    acquire(&state, comic_id, page_id, request).await
}

/// # Errors
/// Rejects invalid tile dimensions, ownership, or exhausted resource handles.
#[tauri::command]
#[specta::specta]
pub async fn get_image_tile(
    state: State<'_, Arc<Harness>>,
    comic_id: String,
    page_id: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> CommandResult<ImageResource> {
    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err(AppError::InvalidInput.into());
    }

    acquire(
        &state,
        comic_id,
        page_id,
        DisplayRequest::Tile {
            x,
            y,
            width,
            height,
        },
    )
    .await
}

/// # Errors
/// Rejects oversized release batches or poisoned resource state.
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri deserializes command arguments into owned invocation values"
)]
#[tauri::command]
#[specta::specta]
pub fn release_image_resources(
    state: State<'_, Arc<Harness>>,
    handles: Vec<String>,
) -> CommandResult<()> {
    Ok(state.resource_registry.release(&handles)?)
}
