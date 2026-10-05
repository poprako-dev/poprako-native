use std::sync::{Arc, Mutex};

use crate::data::editor::{PageEditor, SavePageUnits};
use crate::data::recovery::RecoveryScope;
use crate::data::save_recovery::SaveEvidence;
use crate::implementation::coordinator::SqliteCoordinator;
use crate::part::repository::operation::PersistPageUnits;
use crate::result::AppResult;
use crate::usecase::recovery::{execute, verify_evidence};

/// # Errors
/// Preserves the write halt unless the affected state proves commit or rollback.
pub async fn verify(
    coordinator: &SqliteCoordinator,
    evidence: &SaveEvidence,
) -> AppResult<PageEditor> {
    verify_evidence(coordinator, evidence).await
}

/// # Errors
/// Retains before evidence before writing, even when a failed step also loses its rollback receipt.
pub async fn save(
    coordinator: &SqliteCoordinator,
    snapshot: SavePageUnits,
    evidence: Arc<Mutex<Option<SaveEvidence>>>,
) -> AppResult<PageEditor> {
    let scope = RecoveryScope {
        page_ids: vec![snapshot.page_id.clone()],
        unit_page_ids: vec![snapshot.page_id.clone()],
        unit_ids: snapshot.units.iter().map(|unit| unit.id.clone()).collect(),
        work_position: true,
        ..RecoveryScope::comic(&snapshot.comic_id)
    };

    execute(coordinator, PersistPageUnits { snapshot }, scope, evidence).await
}
