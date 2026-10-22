use poprako_orchestra::Run;
use uuid::Uuid;

use crate::data::comic::{ChapterOrder, ComicInfo, ComicMetadata, MoveChapter, NewComic};
use crate::data::recovery::{EvidenceHandle, RecoveryScope, evidence_handle};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::chapter::Chapter;
use crate::model::comic::Comic;
use crate::part::repository::comic::{
    CreateComic, DeleteComic, ListComicInfos, RelocateChapter, ReorderChapters, UpdateComicMetadata,
};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Rejects invalid pagination or unavailable storage.
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
/// Retains both new identities when the atomic creation outcome is uncertain.
pub async fn create_comic_with_evidence(
    coordinator: &SqliteCoordinator,
    input: NewComic,
    evidence: EvidenceHandle<Chapter>,
) -> AppResult<Chapter> {
    let id = Uuid::new_v4().to_string();

    let chapter_id = Uuid::new_v4().to_string();

    let scope = RecoveryScope {
        comic_ids: vec![id.clone()],
        ..RecoveryScope::chapter(&chapter_id)
    };

    execute(
        coordinator,
        CreateComic {
            id,
            chapter_id,
            input,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Creates the comic and mandatory first chapter atomically.
pub async fn create_comic(coordinator: &SqliteCoordinator, input: NewComic) -> AppResult<Chapter> {
    create_comic_with_evidence(coordinator, input, evidence_handle()).await
}

/// # Errors
/// Rejects missing parents and invalid metadata.
pub async fn update_comic_metadata(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    metadata: ComicMetadata,
) -> AppResult<Comic> {
    execute(
        coordinator,
        UpdateComicMetadata {
            comic_id: comic_id.to_owned(),
            metadata,
        },
        RecoveryScope {
            comic_ids: vec![comic_id.to_owned()],
            ..RecoveryScope::default()
        },
        evidence_handle(),
    )
    .await
}

/// # Errors
/// Captures all descendant identities and work positions before cascading deletion.
pub async fn delete_comic(coordinator: &SqliteCoordinator, comic_id: &str) -> AppResult<()> {
    let scope = RecoveryScope {
        comic_terminology: true,
        comic_ids: vec![comic_id.to_owned()],
        chapter_pages: true,
        chapter_units: true,
        work_position: true,
        ..RecoveryScope::default()
    };

    execute(
        coordinator,
        DeleteComic {
            comic_id: comic_id.to_owned(),
        },
        scope,
        evidence_handle(),
    )
    .await
}

/// # Errors
/// Rejects partial, invalid or stale chapter orders.
pub async fn reorder_chapters(
    coordinator: &SqliteCoordinator,
    input: ChapterOrder,
) -> AppResult<()> {
    let scope = RecoveryScope {
        comic_ids: vec![input.comic_id.clone()],
        ..RecoveryScope::default()
    };

    execute(
        coordinator,
        ReorderChapters { input },
        scope,
        evidence_handle(),
    )
    .await
}

/// # Errors
/// Verifies both parent collections; no page or original image is relocated.
pub async fn move_chapter(
    coordinator: &SqliteCoordinator,
    input: MoveChapter,
) -> AppResult<Chapter> {
    let scope = RecoveryScope {
        comic_ids: vec![input.source_comic_id.clone(), input.target_comic_id.clone()],
        work_position: true,
        ..RecoveryScope::chapter(&input.chapter_id)
    };

    execute(
        coordinator,
        RelocateChapter { input },
        scope,
        evidence_handle(),
    )
    .await
}

/// # Errors
/// Reads the current parent metadata without changing activity timestamps.
pub async fn get_comic(coordinator: &SqliteCoordinator, comic_id: &str) -> AppResult<Comic> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&crate::part::repository::comic::ReadComic {
            comic_id: comic_id.to_owned(),
        })
        .await
}
