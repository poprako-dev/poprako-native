use std::fs;

use image::DynamicImage;
use uuid::Uuid;

use crate::implementation::image::ImagePipeline;
use crate::implementation::resource::ResourceStore;
use crate::implementation::resource::cache::{DisplayCache, DisplayRequest};

#[test]
fn cache_corruption_rebuilds_and_original_stays_identical() -> Result<(), Box<dyn std::error::Error>>
{
    let root = std::env::temp_dir().join(format!("poprako-cache-{}", Uuid::new_v4()));

    ResourceStore::open(&root)?;

    let source = root.join("source.png");

    DynamicImage::new_rgba8(400, 800).save(&source)?;

    let original = fs::read(&source)?;

    let cache = DisplayCache::open(&root)?;

    let pipeline = ImagePipeline::default();

    let id = Uuid::new_v4();

    let preview = cache.get(&pipeline, &source, id, DisplayRequest::Preview)?;

    fs::write(
        root.join("cache").join(format!("v1-{id}-preview.webp")),
        b"broken",
    )?;

    assert_eq!(
        cache.get(&pipeline, &source, id, DisplayRequest::Preview)?,
        preview
    );

    assert_eq!(fs::read(&source)?, original);

    pipeline.release()?;

    fs::remove_dir_all(root)?;

    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_reference_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("poprako-link-{}", Uuid::new_v4()));

    let store = ResourceStore::open(&root)?;

    let comic = Uuid::new_v4();

    let page = Uuid::new_v4();

    let image = Uuid::new_v4();

    let directory = root
        .join("library")
        .join(comic.to_string())
        .join(page.to_string());

    fs::create_dir_all(&directory)?;

    fs::write(root.join("outside.png"), b"outside")?;

    std::os::unix::fs::symlink(
        root.join("outside.png"),
        directory.join(format!("{image}.png")),
    )?;

    assert!(
        store
            .resolve(&format!("library/{comic}/{page}/{image}.png"), comic, page)
            .is_err()
    );

    assert!(
        store
            .remove_unreferenced(&format!("library/{comic}/{page}/{image}.png"), comic, page)
            .is_err()
    );

    fs::remove_file(directory.join(format!("{image}.png")))?;

    fs::remove_dir(&directory)?;

    std::os::unix::fs::symlink(root.join("outside-directory"), &directory)?;

    assert!(
        store
            .remove_unreferenced(&format!("library/{comic}/{page}/{image}.png"), comic, page)
            .is_err()
    );

    fs::remove_dir_all(root)?;

    Ok(())
}

#[test]
fn missing_original_cleanup_requires_valid_ownership() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;

    let store = ResourceStore::open(directory.path())?;

    let comic = Uuid::new_v4();

    let page = Uuid::new_v4();

    let image = Uuid::new_v4();

    let reference = format!("library/{comic}/{page}/{image}.png");

    store.remove_unreferenced(&reference, comic, page)?;

    store.remove_unreferenced(&reference, comic, page)?;

    assert!(
        store
            .remove_unreferenced(&reference, Uuid::new_v4(), page)
            .is_err()
    );

    assert!(
        store
            .remove_unreferenced(
                &format!("library/{comic}/{page}/../outside.png"),
                comic,
                page
            )
            .is_err()
    );

    assert!(
        store
            .remove_unreferenced(&format!("library/{comic}/{page}/{image}.gif"), comic, page)
            .is_err()
    );

    Ok(())
}
