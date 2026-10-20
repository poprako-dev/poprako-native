use std::error::Error;

use tempfile::tempdir;

use crate::data::chapter::ChapterMetadata;
use crate::data::comic::{ChapterOrder, ComicMetadata, NewComic};
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::model::chapter::Chapter;
use crate::result::AppError;
use crate::usecase::{chapter, comic};

pub fn input(title: &str) -> NewComic {
    NewComic {
        comic: ComicMetadata {
            title: title.to_owned(),
            author: format!("{title}作者"),
        },
        chapter_title: "第一章".to_owned(),
    }
}

async fn append(
    coordinator: &SqliteCoordinator,
    first: &Chapter,
    title: &str,
) -> Result<Chapter, Box<dyn Error>> {
    let baseline = chapter::list_chapter_infos(coordinator, &first.comic_id)
        .await?
        .into_iter()
        .map(|info| info.chapter.id)
        .collect();

    Ok(chapter::create_chapter(
        coordinator,
        &first.comic_id,
        baseline,
        ChapterMetadata {
            title: title.to_owned(),
        },
    )
    .await?)
}

#[tokio::test]
async fn first_chapter_is_atomic_and_commit_loss_is_verified() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("database.sqlite")).await?);

    let invalid = NewComic {
        chapter_title: " \t".to_owned(),
        ..input("无效")
    };

    assert_eq!(
        comic::create_comic(&coordinator, invalid).await,
        Err(AppError::InvalidInput)
    );

    assert!(
        comic::list_comic_infos(&coordinator, 0, 50)
            .await?
            .is_empty()
    );

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        comic::create_comic(&coordinator, input("回滚")).await,
        Err(AppError::Storage)
    );

    assert!(
        comic::list_comic_infos(&coordinator, 0, 50)
            .await?
            .is_empty()
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let first = comic::create_comic(&coordinator, input("同名")).await?;

    let second = comic::create_comic(&coordinator, input("同名")).await?;

    assert_ne!(first.comic_id, second.comic_id);

    assert_eq!(comic::list_comic_infos(&coordinator, 0, 50).await?.len(), 2);

    assert!(
        chapter::get_chapter_detail(&coordinator, &first.id)
            .await?
            .pages
            .is_empty()
    );

    chapter::delete_chapter(&coordinator, &first.id).await?;

    assert!(
        chapter::list_chapter_infos(&coordinator, &first.comic_id)
            .await?
            .is_empty()
    );

    append(&coordinator, &first, "重新加章").await?;

    comic::delete_comic(&coordinator, &first.comic_id).await?;

    assert!(
        chapter::get_chapter_detail(&coordinator, &second.id)
            .await
            .is_ok()
    );

    coordinator.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn chapter_order_validates_complete_baselines_and_repairs_deletion()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("database.sqlite")).await?);

    let a = comic::create_comic(&coordinator, input("排序")).await?;

    let b = append(&coordinator, &a, "同名章节").await?;

    let c = append(&coordinator, &a, "同名章节").await?;

    let baseline = vec![a.id.clone(), b.id.clone(), c.id.clone()];

    let reversed = vec![c.id.clone(), b.id.clone(), a.id.clone()];

    let request = ChapterOrder {
        comic_id: a.comic_id.clone(),
        baseline: baseline.clone(),
        ordered_ids: reversed.clone(),
    };

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        comic::reorder_chapters(&coordinator, request.clone()).await,
        Err(AppError::Storage)
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    comic::reorder_chapters(&coordinator, request.clone()).await?;

    assert_eq!(
        comic::reorder_chapters(&coordinator, request).await,
        Err(AppError::Conflict)
    );

    assert_eq!(
        comic::reorder_chapters(
            &coordinator,
            ChapterOrder {
                comic_id: a.comic_id.clone(),
                baseline: reversed.clone(),
                ordered_ids: vec![a.id.clone(), a.id.clone(), b.id.clone()]
            }
        )
        .await,
        Err(AppError::InvalidInput)
    );

    let ordered = chapter::list_chapter_infos(&coordinator, &a.comic_id).await?;

    assert_eq!(
        ordered
            .iter()
            .map(|info| info.chapter.id.clone())
            .collect::<Vec<_>>(),
        reversed
    );

    assert_eq!(ordered[1].chapter.updated_at, b.updated_at);

    chapter::delete_chapter(&coordinator, &b.id).await?;

    let remaining = chapter::list_chapter_infos(&coordinator, &a.comic_id).await?;

    assert_eq!(
        remaining
            .iter()
            .map(|info| info.chapter.index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );

    assert_eq!(remaining[1].chapter.id, a.id);

    coordinator.pool().close().await;

    Ok(())
}
