use crate::data::import::{ExistingChapterImport, NewChapterImport};
use crate::data::recovery::{EvidenceHandle, RecoveryScope, evidence_handle};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::model::chapter::Chapter;
use crate::part::repository::import::{ApplyChapterImport, CreateImportedChapter};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Retains complete new-ID absence and expected imported content for atomic outcome verification.
pub async fn create_chapter_from_import_with_evidence(
    coordinator: &SqliteCoordinator,
    input: NewChapterImport,
    evidence: EvidenceHandle<Chapter>,
) -> AppResult<Chapter> {
    let scope = RecoveryScope {
        comic_ids: vec![input.parent.comic_id().to_owned()],
        chapter_pages: true,
        chapter_units: true,
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
        ..RecoveryScope::chapter(&input.chapter_id)
    };

    execute(
        coordinator,
        CreateImportedChapter { input },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Rejects invalid imported content; any failed page rolls back the entire new chapter.
pub async fn create_chapter_from_import(
    coordinator: &SqliteCoordinator,
    input: NewChapterImport,
) -> AppResult<Chapter> {
    create_chapter_from_import_with_evidence(coordinator, input, evidence_handle()).await
}

/// # Errors
/// Retains complete ordered page and unit content, including work-position repairs.
pub async fn apply_chapter_import_with_evidence(
    coordinator: &SqliteCoordinator,
    input: ExistingChapterImport,
    evidence: EvidenceHandle<Vec<String>>,
) -> AppResult<Vec<String>> {
    let scope = RecoveryScope {
        chapter_pages: true,
        chapter_units: true,
        unit_ids: input
            .pages
            .iter()
            .flat_map(|page| page.iter().map(|unit| unit.id.clone()))
            .collect(),
        work_position: true,
        ..RecoveryScope::chapter(&input.chapter_id)
    };

    execute(coordinator, ApplyChapterImport { input }, scope, evidence).await
}

/// # Errors
/// Rejects stale ordered baselines, page-count mismatches and invalid content atomically.
pub async fn apply_chapter_import(
    coordinator: &SqliteCoordinator,
    input: ExistingChapterImport,
) -> AppResult<Vec<String>> {
    apply_chapter_import_with_evidence(coordinator, input, evidence_handle()).await
}
