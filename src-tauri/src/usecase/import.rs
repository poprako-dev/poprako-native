use crate::data::import::{ExistingComicImport, NewComicImport};
use crate::data::recovery::{EvidenceHandle, RecoveryScope, evidence_handle};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::model::comic::Comic;
use crate::part::repository::import::{ApplyComicImport, CreateImportedComic};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Retains complete new-ID absence and expected imported content for atomic outcome verification.
pub async fn create_comic_from_import_with_evidence(
    coordinator: &SqliteCoordinator,
    input: NewComicImport,
    evidence: EvidenceHandle<Comic>,
) -> AppResult<Comic> {
    let scope = RecoveryScope {
        comic_pages: true,
        comic_units: true,
        page_ids: input
            .pages
            .iter()
            .map(|page| page.page.id.clone())
            .collect(),
        unit_ids: input
            .pages
            .iter()
            .flat_map(|page| page.units.iter().map(|unit| unit.id.clone()))
            .collect(),
        ..RecoveryScope::comic(&input.comic_id)
    };

    execute(coordinator, CreateImportedComic { input }, scope, evidence).await
}

/// # Errors
/// Rejects invalid imported content; any failed page rolls back the entire new comic.
pub async fn create_comic_from_import(
    coordinator: &SqliteCoordinator,
    input: NewComicImport,
) -> AppResult<Comic> {
    create_comic_from_import_with_evidence(coordinator, input, evidence_handle()).await
}

/// # Errors
/// Retains complete ordered page and unit content, including work-position repairs.
pub async fn apply_comic_import_with_evidence(
    coordinator: &SqliteCoordinator,
    input: ExistingComicImport,
    evidence: EvidenceHandle<Vec<String>>,
) -> AppResult<Vec<String>> {
    let scope = RecoveryScope {
        comic_pages: true,
        comic_units: true,
        unit_ids: input
            .pages
            .iter()
            .flat_map(|page| page.iter().map(|unit| unit.id.clone()))
            .collect(),
        work_position: true,
        ..RecoveryScope::comic(&input.comic_id)
    };

    execute(coordinator, ApplyComicImport { input }, scope, evidence).await
}

/// # Errors
/// Rejects stale ordered baselines, page-count mismatches and invalid content atomically.
pub async fn apply_comic_import(
    coordinator: &SqliteCoordinator,
    input: ExistingComicImport,
) -> AppResult<Vec<String>> {
    apply_comic_import_with_evidence(coordinator, input, evidence_handle()).await
}
