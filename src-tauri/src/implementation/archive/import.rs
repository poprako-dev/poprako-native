use uuid::Uuid;

use crate::data::archive::ArchiveComic;
use crate::data::archive::ArchiveUnit;
use crate::data::editor::UnitDraft;
use crate::data::import::{
    ExistingChapterImport, ImportMode, NewChapterImport, PreparedImportPage,
};
use crate::data::page::{NewPage, PageBaseline};
use crate::implementation::archive::source::PreparedArchive;
use crate::implementation::resource::ResourceStore;
use crate::result::{AppError, AppResult};
use crate::value::image::{Image, ImageFormat};

pub struct PublishedArchive {
    pub input: NewChapterImport,
    pub prepared: PreparedArchive,
}

fn draft(unit: &ArchiveUnit) -> UnitDraft {
    UnitDraft {
        id: Uuid::new_v4().to_string(),
        x_coord: unit.x_coord,
        y_coord: unit.y_coord,
        is_bubble: unit.is_bubble,
        is_flagged: unit.is_flagged,
        translated_text: unit.translated_text.clone(),
        proofread_text: unit.proofread_text.clone(),
        is_proofread: unit.is_proofread,
    }
}

fn format(extension: &str) -> AppResult<ImageFormat> {
    match extension {
        "jpg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "webp" => Ok(ImageFormat::Webp),
        "bmp" => Ok(ImageFormat::Bmp),
        _ => Err(AppError::InvalidInput),
    }
}

/// # Errors
/// Publishes immutable files before the caller's single database transaction.
/// Retain this artifact until commit status is known; never clean uncertain commits.
pub fn publish(
    prepared: PreparedArchive,
    metadata: ArchiveComic,
    parent: crate::data::import::ImportParent,
    store: &ResourceStore,
) -> AppResult<PublishedArchive> {
    if prepared.images.len() != prepared.document.pages.len() {
        return Err(AppError::InvalidInput);
    }

    let chapter_id = Uuid::new_v4();

    let mut pages: Vec<PreparedImportPage> = Vec::new();

    for (source, image) in prepared.document.pages.iter().zip(&prepared.images) {
        let page_id = Uuid::new_v4();

        let Ok(reference) = store.publish(image, chapter_id, page_id) else {
            for page in &pages {
                store
                    .remove_unreferenced(
                        &page.page.image.reference,
                        chapter_id,
                        Uuid::parse_str(&page.page.id).map_err(|_| AppError::InvalidInput)?,
                    )
                    .map_err(|_| AppError::Storage)?;
            }

            return Err(AppError::Storage);
        };

        let original_name = source.image.as_ref().map_or_else(
            || source.source_image_path.clone().unwrap_or_default(),
            |image| image.original_name.clone(),
        );

        pages.push(PreparedImportPage {
            page: NewPage {
                id: page_id.to_string(),
                image: Image {
                    reference,
                    original_name,
                    format: format(&image.extension)?,
                    width: image.width,
                    height: image.height,
                },
            },
            units: source.units.iter().map(draft).collect(),
        });
    }

    Ok(PublishedArchive {
        input: NewChapterImport {
            parent,
            chapter_id: chapter_id.to_string(),
            metadata,
            pages,
        },
        prepared,
    })
}

/// # Errors
/// Caller must prove rollback/no commit before setting `committed` to false.
pub fn cleanup(
    published: &PublishedArchive,
    committed: bool,
    store: &ResourceStore,
) -> AppResult<()> {
    if !committed {
        let chapter_id =
            Uuid::parse_str(&published.input.chapter_id).map_err(|_| AppError::InvalidInput)?;

        for page in &published.input.pages {
            let page_id = Uuid::parse_str(&page.page.id).map_err(|_| AppError::InvalidInput)?;

            store
                .remove_unreferenced(&page.page.image.reference, chapter_id, page_id)
                .map_err(|_| AppError::Storage)?;
        }
    }

    for image in &published.prepared.images {
        store
            .discard_prepared(image)
            .map_err(|_| AppError::Storage)?;
    }

    Ok(())
}

/// # Errors
/// Existing targets retain their images and require exact page correspondence.
pub fn existing(
    document: &crate::data::archive::ArchiveDocument,
    chapter_id: String,
    baseline: Vec<PageBaseline>,
    mode: ImportMode,
) -> AppResult<ExistingChapterImport> {
    if baseline.len() != document.pages.len() {
        return Err(AppError::InvalidInput);
    }

    Ok(ExistingChapterImport {
        chapter_id,
        baseline,
        pages: document
            .pages
            .iter()
            .map(|page| page.units.iter().map(draft).collect())
            .collect(),
        mode,
    })
}
