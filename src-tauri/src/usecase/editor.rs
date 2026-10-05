use poprako_orchestra::Run;

use crate::data::editor::{PageEditor, SavePageUnits};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::operation::ReadPageEditor;
use crate::result::AppResult;

pub mod recovery;

/// # Errors
/// Reports missing or invalid page ownership and storage failures.
pub async fn get_page_editor(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    page_id: &str,
) -> AppResult<PageEditor> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    repository
        .run(&ReadPageEditor {
            comic_id: comic_id.to_owned(),
            page_id: page_id.to_owned(),
        })
        .await
}

/// # Errors
/// Rejects invalid or stale snapshots; uncertain commits retain the write halt until verified.
pub async fn save_page_units(
    coordinator: &SqliteCoordinator,
    snapshot: SavePageUnits,
) -> AppResult<PageEditor> {
    recovery::save(
        coordinator,
        snapshot,
        std::sync::Arc::new(std::sync::Mutex::new(None)),
    )
    .await
}

/// # Errors
/// Reports validation or storage errors while retaining exact evidence for unknown commits.
pub async fn save_page_units_with_evidence(
    coordinator: &SqliteCoordinator,
    snapshot: SavePageUnits,
    evidence: std::sync::Arc<std::sync::Mutex<Option<crate::data::save_recovery::SaveEvidence>>>,
) -> AppResult<PageEditor> {
    recovery::save(coordinator, snapshot, evidence).await
}

/// # Errors
/// Returns Storage only for an exact pre-transaction state; an inconclusive result stays halted.
pub async fn verify_page_save(
    coordinator: &SqliteCoordinator,
    evidence: &crate::data::save_recovery::SaveEvidence,
) -> AppResult<PageEditor> {
    recovery::verify(coordinator, evidence).await
}
