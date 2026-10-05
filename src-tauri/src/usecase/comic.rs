use poprako_orchestra::Run;
use uuid::Uuid;

use crate::data::comic::{ComicDetail, ComicInfo, ComicMetadata};
use crate::data::recovery::{EvidenceHandle, RecoveryScope, evidence_handle};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::comic::Comic;
use crate::part::repository::operation::{
    CreateComic, DeleteComic, ListComicInfos, ReadComicDetail, UpdateComicMetadata,
};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Reports invalid pagination or unavailable storage.
pub async fn list_comic_infos(
    coordinator: &SqliteCoordinator,
    offset: u32,
    limit: u32,
) -> AppResult<Vec<ComicInfo>> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ListComicInfos { offset, limit })
        .await
}

/// # Errors
/// Reports invalid identity, missing content, or unavailable storage.
pub async fn get_comic_detail(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
) -> AppResult<ComicDetail> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadComicDetail {
            comic_id: comic_id.to_owned(),
        })
        .await
}

/// # Errors
/// Retains exact creation evidence on commit or rollback uncertainty.
pub async fn create_comic_with_evidence(
    coordinator: &SqliteCoordinator,
    metadata: ComicMetadata,
    evidence: EvidenceHandle<Comic>,
) -> AppResult<Comic> {
    let id = Uuid::new_v4().to_string();

    let scope = RecoveryScope::comic(&id);

    execute(coordinator, CreateComic { id, metadata }, scope, evidence).await
}

/// # Errors
/// Rejects invalid metadata or unavailable storage.
pub async fn create_comic(
    coordinator: &SqliteCoordinator,
    metadata: ComicMetadata,
) -> AppResult<Comic> {
    create_comic_with_evidence(coordinator, metadata, evidence_handle()).await
}

/// # Errors
/// Reports invalid identity, missing content, or uncertain storage outcomes.
pub async fn update_comic_metadata_with_evidence(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    metadata: ComicMetadata,
    evidence: EvidenceHandle<Comic>,
) -> AppResult<Comic> {
    let operation = UpdateComicMetadata {
        comic_id: comic_id.to_owned(),
        metadata,
    };

    execute(
        coordinator,
        operation,
        RecoveryScope::comic(comic_id),
        evidence,
    )
    .await
}

/// # Errors
/// Rejects invalid metadata or unavailable storage.
pub async fn update_comic_metadata(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    metadata: ComicMetadata,
) -> AppResult<Comic> {
    update_comic_metadata_with_evidence(coordinator, comic_id, metadata, evidence_handle()).await
}

/// # Errors
/// Records the deleted aggregate and work position so cascading deletion can be verified exactly.
pub async fn delete_comic_with_evidence(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    evidence: EvidenceHandle<()>,
) -> AppResult<()> {
    let scope = RecoveryScope {
        comic_pages: true,
        comic_units: true,
        work_position: true,
        ..RecoveryScope::comic(comic_id)
    };

    execute(
        coordinator,
        DeleteComic {
            comic_id: comic_id.to_owned(),
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Reports missing content or uncertain storage outcomes.
pub async fn delete_comic(coordinator: &SqliteCoordinator, comic_id: &str) -> AppResult<()> {
    delete_comic_with_evidence(coordinator, comic_id, evidence_handle()).await
}
