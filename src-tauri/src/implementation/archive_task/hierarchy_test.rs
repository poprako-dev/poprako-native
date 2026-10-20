use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use tempfile::tempdir;

use crate::data::archive::ArchiveComic;
use crate::data::archive_task::{ArchiveImportMode, ArchiveTarget, ArchiveTask};
use crate::data::chapter::ChapterMetadata;
use crate::data::comic::{ComicMetadata, NewComic};
use crate::data::search::TextStage;
use crate::harness::Harness;
use crate::implementation::archive_task::picker::ImportSelection;
use crate::implementation::archive_task::{commit, prepare};
use crate::result::AppError;
use crate::usecase::{chapter, comic, export};

async fn prepare_input(
    state: &Arc<Harness>,
    directory: &Path,
    target: ArchiveTarget,
) -> Result<String, Box<dyn Error>> {
    let permit = Arc::clone(&state.resource_task).try_acquire_owned()?;

    let (id, cancel) = state.archive_tasks.create(permit)?;

    prepare::run(
        Arc::clone(state),
        id.clone(),
        ImportSelection {
            source: directory.join("input.lp.txt"),
            images: vec![directory.join("001.png")],
        },
        target,
        TextStage::Translation,
        cancel,
    )
    .await;

    Ok(id)
}

fn metadata() -> ArchiveComic {
    ArchiveComic {
        title: "不覆盖目标漫画".to_owned(),
        subtitle: "导入章".to_owned(),
        author: "来源作者".to_owned(),
    }
}

#[tokio::test]
async fn existing_parent_import_checks_preview_baseline_and_cancellation()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let state = Arc::new(Harness::open(directory.path().join("application"))?);

    let source = directory.path().join("001.png");

    image::RgbImage::new(2, 3).save(&source)?;

    fs::write(
        directory.path().join("input.lp.txt"),
        "1,0\n-\n框内\n框外\n-\nProducer\n>>>>>>>>[001.png]<<<<<<<<\n----------------[1]----------------[0.5,0.5,1]\n译文",
    )?;

    let first = comic::create_comic(
        state.database().await?,
        NewComic {
            comic: ComicMetadata {
                title: "目标漫画".to_owned(),
                author: "目标作者".to_owned(),
            },
            chapter_title: "空首章".to_owned(),
        },
    )
    .await?;

    let target = ArchiveTarget::NewChapter {
        comic_id: first.comic_id.clone(),
        baseline: vec![first.id.clone()],
    };

    let id = prepare_input(&state, directory.path(), target).await?;

    let ArchiveTask::AwaitingConfirmation { preview } = state.archive_tasks.get(&id)? else {
        return Err("missing preview".into());
    };

    assert_eq!(
        preview.target_comic.ok_or("missing parent")?.title,
        "目标漫画"
    );

    let second = chapter::create_chapter(
        state.database().await?,
        &first.comic_id,
        vec![first.id.clone()],
        ChapterMetadata {
            title: "后来添加".to_owned(),
        },
    )
    .await?;

    assert_eq!(
        commit::run(
            Arc::clone(&state),
            id,
            metadata(),
            ArchiveImportMode::FillEmpty
        )
        .await
        .err(),
        Some(AppError::Conflict)
    );

    assert_eq!(
        chapter::list_chapter_infos(state.database().await?, &first.comic_id)
            .await?
            .len(),
        2
    );

    assert_eq!(state.resource_task.available_permits(), 1);

    let target = ArchiveTarget::NewChapter {
        comic_id: first.comic_id.clone(),
        baseline: vec![first.id.clone(), second.id.clone()],
    };

    let cancelled = prepare_input(&state, directory.path(), target.clone()).await?;

    prepare::cancel(Arc::clone(&state), cancelled.clone()).await?;

    assert!(matches!(
        state.archive_tasks.get(&cancelled)?,
        ArchiveTask::Cancelled
    ));

    assert_eq!(
        chapter::list_chapter_infos(state.database().await?, &first.comic_id)
            .await?
            .len(),
        2
    );

    let id = prepare_input(&state, directory.path(), target).await?;

    let completed = commit::run(
        Arc::clone(&state),
        id,
        metadata(),
        ArchiveImportMode::FillEmpty,
    )
    .await?;

    let snapshot = export::snapshot(state.database().await?, &completed.chapter_id).await?;

    assert_eq!(snapshot.chapter.comic_id, first.comic_id);

    assert_eq!(snapshot.chapter.index, 2);

    assert_eq!(snapshot.chapter.title, "导入章");

    assert_eq!(snapshot.comic.title, "目标漫画");

    assert_eq!(snapshot.comic.author, "目标作者");

    assert_eq!(snapshot.pages[0].units[0].translated_text, "译文");

    let original = state.resource.resolve(
        &snapshot.pages[0].page.image.reference,
        uuid::Uuid::parse_str(&completed.chapter_id)?,
        uuid::Uuid::parse_str(&snapshot.pages[0].page.id)?,
    )?;

    assert_eq!(fs::read(original)?, fs::read(source)?);

    Ok(())
}
