use std::fs;
use std::sync::Arc;

use image::RgbImage;
use tempfile::{TempDir, tempdir};

use crate::data::archive_task::{ArchiveImportMode, ArchiveTarget, ArchiveTask};
use crate::data::comic::ComicMetadata;
use crate::data::search::TextStage;
use crate::harness::Harness;
use crate::implementation::archive_task::picker::ImportSelection;
use crate::implementation::archive_task::{commit, export, prepare};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture() -> Result<(TempDir, Arc<Harness>, ImportSelection), Box<dyn std::error::Error>> {
    let directory = tempdir()?;

    let state = Arc::new(Harness::open(directory.path().join("application"))?);

    let image = directory.path().join("001.png");

    let source = directory.path().join("input.lp.txt");

    RgbImage::new(2, 3).save(&image)?;

    fs::write(
        &source,
        "1,0\n-\n框内\n框外\n-\nProducer\n>>>>>>>>[001.png]<<<<<<<<\n----------------[1]----------------[0.5,0.5,1]\n原文",
    )?;

    Ok((
        directory,
        state,
        ImportSelection {
            source,
            images: vec![image],
        },
    ))
}

fn metadata() -> ComicMetadata {
    ComicMetadata {
        title: "项目".to_owned(),
        subtitle: String::new(),
        author: String::new(),
    }
}

#[tokio::test]
async fn prepared_import_commits_once_and_exports_queryable_result() -> TestResult {
    let (directory, state, selection) = fixture()?;

    let permit = Arc::clone(&state.resource_task).try_acquire_owned()?;

    let (id, cancel) = state.archive_tasks.create(permit)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        selection,
        ArchiveTarget::New,
        TextStage::Proofreading,
        cancel,
    )
    .await;

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::AwaitingConfirmation { .. }
    ));

    assert_eq!(state.resource_task.available_permits(), 0);

    let result = commit::run(
        Arc::clone(&state),
        id.clone(),
        metadata(),
        ArchiveImportMode::ReplaceAll,
    )
    .await?;

    assert!(
        commit::run(
            Arc::clone(&state),
            id.clone(),
            metadata(),
            ArchiveImportMode::ReplaceAll
        )
        .await
        .is_err()
    );

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::Completed { .. }
    ));

    let snapshot =
        crate::usecase::export::snapshot(state.database().await?, &result.comic_id).await?;

    assert_eq!(snapshot.pages[0].units[0].proofread_text, "原文");

    assert!(snapshot.pages[0].units[0].translated_text.is_empty());

    assert_eq!(state.resource_task.available_permits(), 1);

    state.archive_tasks.acknowledge(&id)?;

    let (export_id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    let target = directory.path().join("export.zip");

    export::run(
        Arc::clone(&state),
        export_id.clone(),
        result.comic_id,
        target.clone(),
        true,
        false,
        cancel,
    )
    .await;

    assert!(target.exists());

    assert!(matches!(
        state.archive_tasks.get(&export_id)?,
        ArchiveTask::Completed { .. }
    ));

    assert_eq!(state.resource_task.available_permits(), 1);

    Ok(())
}

#[tokio::test]
async fn cancelling_ready_preview_releases_budget_without_database_changes() -> TestResult {
    let (_directory, state, selection) = fixture()?;

    let image = selection.images[0].clone();

    let (id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        selection,
        ArchiveTarget::New,
        TextStage::Translation,
        cancel,
    )
    .await;

    prepare::cancel(Arc::clone(&state), id.clone()).await?;

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::Cancelled
    ));

    assert_eq!(state.resource_task.available_permits(), 1);

    assert!(image.exists());

    assert!(
        commit::run(
            Arc::clone(&state),
            id.clone(),
            metadata(),
            ArchiveImportMode::ReplaceAll
        )
        .await
        .is_err()
    );

    state.archive_tasks.acknowledge(&id)?;

    Ok(())
}

#[tokio::test]
async fn malformed_source_finishes_failure_and_releases_budget() -> TestResult {
    let (_directory, state, selection) = fixture()?;

    fs::write(&selection.source, "not LabelPlus")?;

    let (id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        selection,
        ArchiveTarget::New,
        TextStage::Translation,
        cancel,
    )
    .await;

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::Failed {
            commit_uncertain: false,
            ..
        }
    ));

    assert_eq!(state.resource_task.available_permits(), 1);

    state.archive_tasks.acknowledge(&id)?;

    Ok(())
}

#[tokio::test]
async fn existing_import_rejects_stale_preview_without_overwriting_edit() -> TestResult {
    let (_directory, state, selection) = fixture()?;

    let existing_source = selection.source.clone();

    let (id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        selection,
        ArchiveTarget::New,
        TextStage::Translation,
        cancel,
    )
    .await;

    let created = commit::run(
        Arc::clone(&state),
        id,
        metadata(),
        ArchiveImportMode::ReplaceAll,
    )
    .await?;

    let (id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        ImportSelection {
            source: existing_source,
            images: Vec::new(),
        },
        ArchiveTarget::Existing {
            comic_id: created.comic_id.clone(),
        },
        TextStage::Proofreading,
        cancel,
    )
    .await;

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::AwaitingConfirmation { .. }
    ));

    let snapshot =
        crate::usecase::export::snapshot(state.database().await?, &created.comic_id).await?;

    let page = &snapshot.pages[0];

    let mut unit = crate::data::editor::UnitDraft::from(&page.units[0]);

    unit.translated_text = "用户新编辑".to_owned();

    crate::usecase::editor::save_page_units(
        state.database().await?,
        crate::data::editor::SavePageUnits {
            comic_id: created.comic_id.clone(),
            page_id: page.page.id.clone(),
            expected_revision: page.page.unit_revision,
            units: vec![unit],
        },
    )
    .await?;

    assert!(matches!(
        commit::run(
            Arc::clone(&state),
            id.clone(),
            metadata(),
            ArchiveImportMode::ReplaceAll
        )
        .await,
        Err(crate::result::AppError::Conflict)
    ));

    let snapshot =
        crate::usecase::export::snapshot(state.database().await?, &created.comic_id).await?;

    assert_eq!(snapshot.pages[0].units[0].translated_text, "用户新编辑");

    assert_eq!(state.resource_task.available_permits(), 1);

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::Failed {
            commit_uncertain: false,
            ..
        }
    ));

    Ok(())
}

#[tokio::test]
async fn cancellation_before_preparation_and_uncertain_commit_keep_distinct_boundaries()
-> TestResult {
    let (_directory, state, selection) = fixture()?;

    let (id, cancel) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    state.archive_tasks.cancel(&id)?;

    prepare::run(
        Arc::clone(&state),
        id.clone(),
        selection,
        ArchiveTarget::New,
        TextStage::Translation,
        cancel,
    )
    .await;

    assert!(matches!(
        state.archive_tasks.get(&id)?,
        ArchiveTask::Cancelled
    ));

    assert_eq!(state.resource_task.available_permits(), 1);

    let (uncertain, _) = state
        .archive_tasks
        .create(Arc::clone(&state.resource_task).try_acquire_owned()?)?;

    state.archive_tasks.update(&uncertain, |record| {
        record.status = ArchiveTask::Failed {
            message: "核实提交中".to_owned(),
            commit_uncertain: true,
        };

        Ok(())
    })?;

    assert!(state.archive_tasks.cancel(&uncertain).is_err());

    assert!(state.archive_tasks.acknowledge(&uncertain).is_err());

    assert_eq!(state.resource_task.available_permits(), 0);

    Ok(())
}
