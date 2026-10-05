use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use image::DynamicImage;
use tempfile::TempDir;
use uuid::Uuid;

use crate::data::comic::ComicMetadata;
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::image_import::{ImageImportTarget, ImageTaskPhase};
use crate::data::page::PageBaseline;
use crate::harness::Harness;
use crate::implementation::coordinator::CommitFault;
use crate::implementation::image_task::{commit, name, prepare, recovery};
use crate::model::page::Page;
use crate::result::{AppError, AppResult};
use crate::usecase::{comic, editor};

pub async fn setup() -> Result<(TempDir, Arc<Harness>, String), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;

    let harness = Arc::new(Harness::open(directory.path().join("managed"))?);

    let comic = comic::create_comic(
        harness.database().await?,
        ComicMetadata {
            title: "Image workflow".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
    )
    .await?;

    Ok((directory, harness, comic.id))
}

fn baseline(page: &Page) -> PageBaseline {
    PageBaseline {
        id: page.id.clone(),
        unit_revision: page.unit_revision,
        image_reference: page.image.reference.clone(),
    }
}

async fn wait(harness: &Harness, task_id: &str) -> AppResult<ImageTaskPhase> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let phase = harness.image_task.status(task_id)?.phase;

            if !matches!(
                phase,
                ImageTaskPhase::Selecting | ImageTaskPhase::Preparing | ImageTaskPhase::Stopping
            ) {
                return Ok(phase);
            }

            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .map_err(|_| AppError::Busy)?
}

pub async fn stage(
    harness: &Arc<Harness>,
    target: ImageImportTarget,
    paths: Vec<std::path::PathBuf>,
) -> AppResult<String> {
    let status = prepare::begin(harness, target).await?;

    crate::usecase::image_import::start_preparation(
        Arc::clone(harness),
        status.task_id.clone(),
        paths,
    );

    wait(harness, &status.task_id).await?;

    Ok(status.task_id)
}

#[tokio::test]
async fn append_preserves_bytes_and_natural_order() -> Result<(), Box<dyn Error>> {
    let (directory, harness, comic_id) = setup().await?;

    let a = directory.path().join("10.png");

    let b = directory.path().join("2.png");

    DynamicImage::new_rgba8(8, 12).save(&a)?;

    DynamicImage::new_rgb8(12, 8).save(&b)?;

    let expected = std::fs::read(&b)?;

    let task_id = stage(
        &harness,
        ImageImportTarget::Append {
            comic_id: comic_id.clone(),
            baseline: Vec::new(),
        },
        vec![a, b],
    )
    .await?;

    let status = harness.image_task.status(&task_id)?;

    assert_eq!(status.phase, ImageTaskPhase::Ready);

    assert_eq!(status.previews[0].original_name, "2.png");

    assert!(
        harness
            .image_task
            .preview(
                &status.previews[0].handle,
                &harness.resource,
                &harness.image
            )
            .is_ok()
    );

    let pages = commit::confirm(Arc::clone(&harness), task_id.clone(), false).await?;

    let path = harness.resource.resolve(
        &pages[0].image.reference,
        Uuid::parse_str(&comic_id)?,
        Uuid::parse_str(&pages[0].id)?,
    )?;

    assert_eq!(std::fs::read(path)?, expected);

    assert_eq!(
        harness.image_task.status(&task_id)?.phase,
        ImageTaskPhase::Completed
    );

    assert_eq!(harness.resource_task.available_permits(), 1);

    assert!(
        harness
            .image_task
            .preview(
                &status.previews[0].handle,
                &harness.resource,
                &harness.image
            )
            .is_err()
    );

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn stale_revision_rejects_whole_image_replacement() -> Result<(), Box<dyn Error>> {
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgba8(8, 12).save(&source)?;

    let task = stage(
        &harness,
        ImageImportTarget::Append {
            comic_id: comic_id.clone(),
            baseline: Vec::new(),
        },
        vec![source.clone()],
    )
    .await?;

    let pages = commit::confirm(Arc::clone(&harness), task, false).await?;

    let page = &pages[0];

    let replacement = stage(
        &harness,
        ImageImportTarget::Replace {
            comic_id: comic_id.clone(),
            baseline: baseline(page),
        },
        vec![source],
    )
    .await?;

    editor::save_page_units(
        harness.database().await?,
        SavePageUnits {
            comic_id: comic_id.clone(),
            page_id: page.id.clone(),
            expected_revision: 0,
            units: vec![UnitDraft {
                id: Uuid::new_v4().to_string(),
                x_coord: 0.5,
                y_coord: 0.5,
                is_bubble: true,
                is_flagged: false,
                translated_text: "New draft".to_owned(),
                proofread_text: String::new(),
                is_proofread: false,
            }],
        },
    )
    .await?;

    assert_eq!(
        commit::confirm(Arc::clone(&harness), replacement.clone(), true).await,
        Err(AppError::Conflict)
    );

    let current = editor::get_page_editor(harness.database().await?, &comic_id, &page.id).await?;

    assert_eq!(current.page.image, page.image);

    assert_eq!(current.units.len(), 1);

    assert_eq!(
        harness.image_task.status(&replacement)?.phase,
        ImageTaskPhase::Failed
    );

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn partial_prepare_failure_and_early_cancel_release_resources() -> Result<(), Box<dyn Error>>
{
    let (directory, harness, comic_id) = setup().await?;

    let good = directory.path().join("1.png");

    let bad = directory.path().join("2.png");

    DynamicImage::new_rgba8(8, 12).save(&good)?;

    std::fs::write(&bad, b"not an image")?;

    let target = ImageImportTarget::Append {
        comic_id,
        baseline: Vec::new(),
    };

    let task_id = stage(&harness, target.clone(), vec![good.clone(), bad]).await?;

    assert_eq!(
        harness.image_task.status(&task_id)?.phase,
        ImageTaskPhase::Failed
    );

    assert_eq!(harness.resource_task.available_permits(), 1);

    assert!(
        std::fs::read_dir(harness.resource.root().join("staging").join(&task_id))?
            .next()
            .is_none()
    );

    let permit = Arc::clone(&harness.resource_task).try_acquire_owned()?;

    let status = harness.image_task.begin(target, permit)?;

    harness.image_task.request_stop(&status.task_id)?;

    crate::usecase::image_import::start_preparation(
        Arc::clone(&harness),
        status.task_id.clone(),
        vec![good],
    );

    assert_eq!(
        wait(&harness, &status.task_id).await?,
        ImageTaskPhase::Cancelled
    );

    assert_eq!(harness.resource_task.available_permits(), 1);

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn unknown_commit_preserves_published_originals_and_blocks_gc() -> Result<(), Box<dyn Error>>
{
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgba8(8, 12).save(&source)?;

    let task_id = stage(
        &harness,
        ImageImportTarget::Append {
            comic_id: comic_id.clone(),
            baseline: Vec::new(),
        },
        vec![source],
    )
    .await?;

    harness
        .database()
        .await?
        .inject_commit_fault(CommitFault::LostCommitted);

    assert_eq!(
        commit::confirm(Arc::clone(&harness), task_id.clone(), false).await,
        Err(AppError::CommitUncertain)
    );

    assert_eq!(
        harness.image_task.status(&task_id)?.phase,
        ImageTaskPhase::Uncertain
    );

    assert_eq!(harness.resource_task.available_permits(), 0);

    assert_eq!(
        recovery::recover(Arc::clone(&harness)).await,
        Err(AppError::Busy)
    );

    let detail = comic::get_comic_detail(harness.database().await?, &comic_id).await?;

    let page = &detail.pages[0].page;

    assert!(
        harness
            .resource
            .resolve(
                &page.image.reference,
                Uuid::parse_str(&comic_id)?,
                Uuid::parse_str(&page.id)?
            )
            .is_ok()
    );

    harness.database().await?.pool().close().await;

    Ok(())
}

#[test]
fn normalized_natural_sort_is_deterministic() -> Result<(), Box<dyn Error>> {
    let names = [
        "10.png",
        "001.png",
        "01.png",
        "1.png",
        "2.png",
        "é.png",
        "e\u{301}.png",
        "999999999999999999999999.png",
    ];

    let sorted = name::sort_sources(names.iter().map(std::path::PathBuf::from).collect())?;

    let names: Vec<_> = sorted.into_iter().map(|(_, name)| name).collect();

    assert_eq!(
        names,
        [
            "1.png",
            "01.png",
            "001.png",
            "2.png",
            "10.png",
            "999999999999999999999999.png",
            "e\u{301}.png",
            "é.png"
        ]
    );

    Ok(())
}
