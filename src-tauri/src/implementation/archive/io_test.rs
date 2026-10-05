use std::fs;
use std::io::Cursor;

use image::RgbImage;
use tempfile::tempdir;

use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchiveImage, ArchivePage};
use crate::data::comic::ComicMetadata;
use crate::data::search::TextStage;
use crate::implementation::archive::{destination, import, package, source};
use crate::implementation::image::ImagePipeline;
use crate::implementation::resource::ResourceStore;
use crate::value::image::ImageFormat;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn document() -> ArchiveDocument {
    ArchiveDocument {
        comic: ArchiveComic {
            title: "项目".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
        pages: Vec::new(),
        warnings: Vec::new(),
    }
}

#[test]
fn destination_preserves_existing_file_without_overwrite_confirmation() -> TestResult {
    let directory = tempdir()?;

    let target = directory.path().join("export.zip");

    fs::write(&target, b"previous result")?;

    assert!(
        destination::export(&target, &document(), false, false, |_| Err::<
            std::fs::File,
            _,
        >(
            crate::result::AppError::NotFound
        ))
        .is_err()
    );

    assert_eq!(fs::read(&target)?, b"previous result");

    destination::export(&target, &document(), false, true, |_| {
        Err::<std::fs::File, _>(crate::result::AppError::NotFound)
    })?;

    let mut package = package::open(fs::File::open(&target)?)?;

    assert_eq!(
        package::read_document(&mut package, &TextStage::Translation)?,
        document()
    );

    assert_eq!(fs::read_dir(directory.path())?.count(), 1);

    Ok(())
}

#[test]
fn zip_import_retains_original_and_cleans_rolled_back_publication() -> TestResult {
    let directory = tempdir()?;

    let image_path = directory.path().join("source.png");

    RgbImage::new(2, 3).save(&image_path)?;

    let original = fs::read(&image_path)?;

    let mut document = document();

    document.pages.push(ArchivePage {
        index: 0,
        source_image_path: Some("images/001.png".to_owned()),
        image: Some(ArchiveImage {
            path: "images/001.png".to_owned(),
            original_name: "source.png".to_owned(),
            format: ImageFormat::Png,
            width: 2,
            height: 3,
        }),
        units: Vec::new(),
    });

    let source_path = directory.path().join("input.zip");

    let writer = package::write_package(Cursor::new(Vec::new()), &document, true, |_| {
        Ok(Cursor::new(original.clone()))
    })?;

    fs::write(&source_path, writer.into_inner())?;

    let store = ResourceStore::open(&directory.path().join("resources"))?;

    let pipeline = ImagePipeline::default();

    let prepared = source::prepare(
        &source_path,
        &[],
        directory.path(),
        &store,
        &pipeline,
        &TextStage::Translation,
    )?;

    let published = import::publish(
        prepared,
        ComicMetadata {
            title: "新项目".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
        &store,
    )?;

    let page = &published.input.pages[0].page;

    let comic_id = uuid::Uuid::parse_str(&published.input.comic_id)?;

    let page_id = uuid::Uuid::parse_str(&page.id)?;

    let managed = store.resolve(&page.image.reference, comic_id, page_id)?;

    assert_eq!(fs::read(&managed)?, original);

    import::cleanup(&published, false, &store)?;

    assert!(!managed.exists());

    assert_eq!(fs::read(&image_path)?, original);

    Ok(())
}
