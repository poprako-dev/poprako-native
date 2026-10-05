use std::error::Error;
use std::io::Read;
use std::sync::Arc;

use image::DynamicImage;
use uuid::Uuid;

use crate::data::image_import::ImageImportTarget;
use crate::implementation::image_task::test::{setup, stage};
use crate::implementation::image_task::{commit, recovery};

#[tokio::test]
async fn recovery_preserves_references_and_unknown_files() -> Result<(), Box<dyn Error>> {
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgb8(8, 12).save(&source)?;

    let original = std::fs::read(&source)?;

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

    let page_id = Uuid::parse_str(&pages[0].id)?;

    let comic_id = Uuid::parse_str(&comic_id)?;

    let orphan = harness
        .resource
        .prepare(&source, Uuid::new_v4(), &harness.image)?;

    let orphan_reference = harness.resource.publish(&orphan, comic_id, page_id)?;

    let orphan_path = harness
        .resource
        .resolve(&orphan_reference, comic_id, page_id)?;

    let unknown = harness
        .resource
        .root()
        .join("library")
        .join("unknown-user-file");

    std::fs::write(&unknown, b"keep me")?;

    recovery::recover(Arc::clone(&harness)).await?;

    assert!(!orphan_path.exists());

    assert_eq!(std::fs::read(&unknown)?, b"keep me");

    assert_eq!(std::fs::read(&source)?, original);

    assert_eq!(
        std::fs::read(
            harness
                .resource
                .resolve(&pages[0].image.reference, comic_id, page_id)?
        )?,
        original
    );

    assert!(
        !harness
            .resource
            .root()
            .join("staging")
            .join(orphan.task_id.to_string())
            .exists()
    );

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn original_lease_blocks_removal_until_export_reader_finishes() -> Result<(), Box<dyn Error>>
{
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgb8(8, 12).save(&source)?;

    let original = std::fs::read(&source)?;

    let prepared = harness
        .resource
        .prepare(&source, Uuid::new_v4(), &harness.image)?;

    let comic_id = Uuid::parse_str(&comic_id)?;

    let page_id = Uuid::new_v4();

    let reference = harness.resource.publish(&prepared, comic_id, page_id)?;

    let mut lease = harness
        .resource
        .open_original(&reference, comic_id, page_id)?;

    assert!(
        harness
            .resource
            .remove_unreferenced(&reference, comic_id, page_id)
            .is_err()
    );

    let mut bytes = Vec::new();

    lease.file.read_to_end(&mut bytes)?;

    assert_eq!(bytes, original);

    drop(lease);

    harness
        .resource
        .remove_unreferenced(&reference, comic_id, page_id)?;

    assert!(
        harness
            .resource
            .resolve(&reference, comic_id, page_id)
            .is_err()
    );

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn database_unavailable_never_turns_into_delete_all() -> Result<(), Box<dyn Error>> {
    let (directory, harness, comic_id) = setup().await?;

    let source = directory.path().join("source.png");

    DynamicImage::new_rgb8(8, 12).save(&source)?;

    let prepared = harness
        .resource
        .prepare(&source, Uuid::new_v4(), &harness.image)?;

    let comic_id = Uuid::parse_str(&comic_id)?;

    let page_id = Uuid::new_v4();

    let reference = harness.resource.publish(&prepared, comic_id, page_id)?;

    let path = harness.resource.resolve(&reference, comic_id, page_id)?;

    harness.database().await?.pool().close().await;

    assert!(recovery::recover(Arc::clone(&harness)).await.is_err());

    assert!(path.exists());

    assert!(
        harness
            .resource
            .root()
            .join("staging")
            .join(prepared.task_id.to_string())
            .exists()
    );

    Ok(())
}
