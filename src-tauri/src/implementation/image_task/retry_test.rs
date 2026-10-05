use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use image::DynamicImage;
use tempfile::TempDir;
use uuid::Uuid;

use crate::data::image_import::{ImageImportTarget, ImageTaskPhase};
use crate::data::page::{PageBaseline, RemovePages};
use crate::harness::Harness;
use crate::implementation::image_task::test::{setup, stage};
use crate::implementation::image_task::{cancel, commit};
use crate::implementation::resource::cache::DisplayRequest;
use crate::implementation::resource::registry::ImageGrant;
use crate::model::page::Page;
use crate::result::AppError;
use crate::usecase::page;

fn baseline(page: &Page) -> PageBaseline {
    PageBaseline {
        id: page.id.clone(),
        unit_revision: page.unit_revision,
        image_reference: page.image.reference.clone(),
    }
}

struct Replacement {
    _directory: TempDir,
    harness: Arc<Harness>,
    comic_id: String,
    source: PathBuf,
    old_path: PathBuf,
    old: Page,
    current: Page,
    task_id: String,
    grant: String,
}

async fn replacement() -> Result<Replacement, Box<dyn Error>> {
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgb8(8, 12).save(&source)?;

    let initial = stage(
        &harness,
        ImageImportTarget::Append {
            comic_id: comic_id.clone(),
            baseline: Vec::new(),
        },
        vec![source.clone()],
    )
    .await?;

    let pages = commit::confirm(Arc::clone(&harness), initial, false).await?;

    let old = &pages[0];

    let old_path = harness.resource.resolve(
        &old.image.reference,
        Uuid::parse_str(&comic_id)?,
        Uuid::parse_str(&old.id)?,
    )?;

    let grant = harness.resource_registry.acquire(ImageGrant {
        comic_id: Uuid::parse_str(&comic_id)?,
        page_id: Uuid::parse_str(&old.id)?,
        reference: old.image.reference.clone(),
        request: DisplayRequest::Thumbnail,
    })?;

    let replacement = stage(
        &harness,
        ImageImportTarget::Replace {
            comic_id: comic_id.clone(),
            baseline: baseline(old),
        },
        vec![source.clone()],
    )
    .await?;

    let replaced = commit::confirm(Arc::clone(&harness), replacement.clone(), false).await?;

    assert_eq!(
        harness.image_task.status(&replacement)?.phase,
        ImageTaskPhase::Completed
    );

    assert!(harness.image_task.status(&replacement)?.cleanup_pending);

    assert!(old_path.exists());

    Ok(Replacement {
        _directory: directory,
        harness,
        comic_id,
        source,
        old_path,
        old: old.clone(),
        current: replaced[0].clone(),
        task_id: replacement,
        grant,
    })
}

#[tokio::test]
async fn released_thumbnail_allows_following_append_and_replace() -> Result<(), Box<dyn Error>> {
    let fixture = replacement().await?;

    let Replacement {
        harness,
        comic_id,
        source,
        old_path,
        old,
        current,
        grant,
        ..
    } = &fixture;

    let target = ImageImportTarget::Append {
        comic_id: comic_id.clone(),
        baseline: vec![baseline(current)],
    };

    assert!(matches!(
        stage(harness, target.clone(), vec![source.clone()]).await,
        Err(AppError::Busy)
    ));

    assert!(old_path.exists());

    harness
        .resource_registry
        .release(std::slice::from_ref(grant))?;

    let next = stage(harness, target, vec![source.clone()]).await?;

    assert!(!old_path.exists());

    cancel::cancel(Arc::clone(harness), next).await?;

    let next = stage(
        harness,
        ImageImportTarget::Replace {
            comic_id: comic_id.clone(),
            baseline: baseline(current),
        },
        vec![source.clone()],
    )
    .await?;

    let again = commit::confirm(Arc::clone(harness), next, false).await?;

    assert_eq!(again[0].id, old.id);

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn unavailable_database_retains_pending_originals() -> Result<(), Box<dyn Error>> {
    let fixture = replacement().await?;

    fixture
        .harness
        .resource_registry
        .release(std::slice::from_ref(&fixture.grant))?;

    fixture.harness.database().await?.pool().close().await;

    assert!(
        cancel::cancel(Arc::clone(&fixture.harness), fixture.task_id.clone())
            .await
            .is_err()
    );

    assert!(fixture.old_path.exists());

    assert!(
        fixture
            .harness
            .image_task
            .status(&fixture.task_id)?
            .cleanup_pending
    );

    Ok(())
}

#[tokio::test]
async fn already_deleted_old_image_can_finish_retained_cleanup() -> Result<(), Box<dyn Error>> {
    let fixture = replacement().await?;

    fixture
        .harness
        .resource_registry
        .release(std::slice::from_ref(&fixture.grant))?;

    let lease = fixture.harness.resource.open_original(
        &fixture.old.image.reference,
        Uuid::parse_str(&fixture.comic_id)?,
        Uuid::parse_str(&fixture.old.id)?,
    )?;

    std::fs::remove_file(&fixture.old_path)?;

    assert!(
        cancel::cancel(Arc::clone(&fixture.harness), fixture.task_id.clone())
            .await
            .is_err()
    );

    assert!(
        fixture
            .harness
            .image_task
            .status(&fixture.task_id)?
            .cleanup_pending
    );

    drop(lease);

    let status = cancel::cancel(Arc::clone(&fixture.harness), fixture.task_id.clone()).await?;

    assert!(!status.cleanup_pending);

    let status = cancel::cancel(Arc::clone(&fixture.harness), fixture.task_id.clone()).await?;

    assert!(!status.cleanup_pending);

    fixture.harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn retry_keeps_protected_cleanup_pending_after_page_deletion() -> Result<(), Box<dyn Error>> {
    let fixture = replacement().await?;

    assert!(matches!(
        cancel::cancel(Arc::clone(&fixture.harness), fixture.task_id.clone()).await,
        Err(AppError::Busy)
    ));

    assert!(
        fixture
            .harness
            .image_task
            .status(&fixture.task_id)?
            .cleanup_pending
    );

    page::remove_pages(
        fixture.harness.database().await?,
        RemovePages {
            comic_id: fixture.comic_id.clone(),
            baseline: vec![baseline(&fixture.current)],
            page_ids: vec![fixture.current.id.clone()],
        },
    )
    .await?;

    assert!(fixture.old_path.exists());

    fixture
        .harness
        .resource_registry
        .release(std::slice::from_ref(&fixture.grant))?;

    let next = stage(
        &fixture.harness,
        ImageImportTarget::Append {
            comic_id: fixture.comic_id.clone(),
            baseline: Vec::new(),
        },
        vec![fixture.source.clone()],
    )
    .await?;

    assert!(!fixture.old_path.exists());

    cancel::cancel(Arc::clone(&fixture.harness), next).await?;

    fixture.harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn transient_cleanup_failure_retains_resources_until_next_import()
-> Result<(), Box<dyn Error>> {
    let fixture = replacement().await?;

    fixture
        .harness
        .resource_registry
        .release(std::slice::from_ref(&fixture.grant))?;

    let bytes = std::fs::read(&fixture.old_path)?;

    std::fs::remove_file(&fixture.old_path)?;

    std::fs::create_dir(&fixture.old_path)?;

    let target = ImageImportTarget::Append {
        comic_id: fixture.comic_id.clone(),
        baseline: vec![baseline(&fixture.current)],
    };

    assert!(
        stage(
            &fixture.harness,
            target.clone(),
            vec![fixture.source.clone()]
        )
        .await
        .is_err()
    );

    let status = fixture.harness.image_task.status(&fixture.task_id)?;

    assert_eq!(status.phase, ImageTaskPhase::Completed);

    assert!(status.cleanup_pending);

    assert_eq!(fixture.harness.resource_task.available_permits(), 1);

    std::fs::remove_dir(&fixture.old_path)?;

    std::fs::write(&fixture.old_path, bytes)?;

    let lease = fixture.harness.resource.open_original(
        &fixture.old.image.reference,
        Uuid::parse_str(&fixture.comic_id)?,
        Uuid::parse_str(&fixture.old.id)?,
    )?;

    assert!(
        stage(
            &fixture.harness,
            target.clone(),
            vec![fixture.source.clone()]
        )
        .await
        .is_err()
    );

    assert!(fixture.old_path.exists());

    assert!(
        fixture
            .harness
            .image_task
            .status(&fixture.task_id)?
            .cleanup_pending
    );

    drop(lease);

    let next = stage(&fixture.harness, target, vec![fixture.source.clone()]).await?;

    assert!(!fixture.old_path.exists());

    cancel::cancel(Arc::clone(&fixture.harness), next).await?;

    fixture.harness.database().await?.pool().close().await;

    Ok(())
}
