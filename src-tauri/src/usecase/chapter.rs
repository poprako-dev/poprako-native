use poprako_orchestra::Run;
use uuid::Uuid;

use crate::data::chapter::{ChapterDetail, ChapterInfo, ChapterMetadata};
use crate::data::recovery::{EvidenceHandle, RecoveryScope, evidence_handle};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::chapter::Chapter;
use crate::part::repository::operation::{
    CreateChapter, DeleteChapter, ListChapterInfos, ReadChapterDetail, UpdateChapterMetadata,
};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Reports invalid pagination or unavailable storage.
pub async fn list_chapter_infos(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
) -> AppResult<Vec<ChapterInfo>> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ListChapterInfos {
            comic_id: comic_id.to_owned(),
        })
        .await
}

/// # Errors
/// Reports invalid identity, missing content, or unavailable storage.
pub async fn get_chapter_detail(
    coordinator: &SqliteCoordinator,
    chapter_id: &str,
) -> AppResult<ChapterDetail> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadChapterDetail {
            chapter_id: chapter_id.to_owned(),
        })
        .await
}

/// # Errors
/// Retains exact creation evidence on commit or rollback uncertainty.
pub async fn create_chapter_with_evidence(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    baseline: Vec<String>,
    metadata: ChapterMetadata,
    evidence: EvidenceHandle<Chapter>,
) -> AppResult<Chapter> {
    let id = Uuid::new_v4().to_string();

    let scope = RecoveryScope {
        comic_ids: vec![comic_id.to_owned()],
        ..RecoveryScope::chapter(&id)
    };

    execute(
        coordinator,
        CreateChapter {
            id,
            comic_id: comic_id.to_owned(),
            baseline,
            metadata,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Rejects invalid metadata or unavailable storage.
pub async fn create_chapter(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    baseline: Vec<String>,
    metadata: ChapterMetadata,
) -> AppResult<Chapter> {
    create_chapter_with_evidence(coordinator, comic_id, baseline, metadata, evidence_handle()).await
}

/// # Errors
/// Reports invalid identity, missing content, or uncertain storage outcomes.
pub async fn update_chapter_metadata_with_evidence(
    coordinator: &SqliteCoordinator,
    chapter_id: &str,
    metadata: ChapterMetadata,
    evidence: EvidenceHandle<Chapter>,
) -> AppResult<Chapter> {
    let operation = UpdateChapterMetadata {
        chapter_id: chapter_id.to_owned(),
        metadata,
    };

    execute(
        coordinator,
        operation,
        RecoveryScope::chapter(chapter_id),
        evidence,
    )
    .await
}

/// # Errors
/// Rejects invalid metadata or unavailable storage.
pub async fn update_chapter_metadata(
    coordinator: &SqliteCoordinator,
    chapter_id: &str,
    metadata: ChapterMetadata,
) -> AppResult<Chapter> {
    update_chapter_metadata_with_evidence(coordinator, chapter_id, metadata, evidence_handle())
        .await
}

/// # Errors
/// Records the deleted aggregate and work position so cascading deletion can be verified exactly.
pub async fn delete_chapter_with_evidence(
    coordinator: &SqliteCoordinator,
    chapter_id: &str,
    evidence: EvidenceHandle<()>,
) -> AppResult<()> {
    let detail = get_chapter_detail(coordinator, chapter_id).await?;

    let scope = RecoveryScope {
        comic_ids: vec![detail.chapter.comic_id],
        chapter_pages: true,
        chapter_units: true,
        work_position: true,
        ..RecoveryScope::chapter(chapter_id)
    };

    execute(
        coordinator,
        DeleteChapter {
            chapter_id: chapter_id.to_owned(),
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Reports missing content or uncertain storage outcomes.
pub async fn delete_chapter(coordinator: &SqliteCoordinator, chapter_id: &str) -> AppResult<()> {
    delete_chapter_with_evidence(coordinator, chapter_id, evidence_handle()).await
}
