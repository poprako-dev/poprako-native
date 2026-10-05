use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchiveImage, ArchivePage, ArchiveUnit};
use crate::data::export::ExportSnapshot;
use crate::value::image::ImageFormat;

fn extension(format: &ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        ImageFormat::Webp => "webp",
        ImageFormat::Bmp => "bmp",
    }
}

#[must_use]
pub fn document(snapshot: &ExportSnapshot) -> ArchiveDocument {
    let width = snapshot.pages.len().to_string().len().max(3);

    let pages = snapshot
        .pages
        .iter()
        .enumerate()
        .map(|(index, editor)| {
            let image = &editor.page.image;

            let path = format!("images/{:0width$}.{}", index + 1, extension(&image.format));

            let units = editor
                .units
                .iter()
                .map(|unit| ArchiveUnit {
                    index: unit.index,
                    x_coord: unit.x_coord,
                    y_coord: unit.y_coord,
                    is_bubble: unit.is_bubble,
                    is_flagged: unit.is_flagged,
                    translated_text: unit.translated_text.clone(),
                    proofread_text: unit.proofread_text.clone(),
                    is_proofread: unit.is_proofread,
                })
                .collect();

            ArchivePage {
                index: editor.page.index,
                image: Some(ArchiveImage {
                    path: path.clone(),
                    original_name: image.original_name.clone(),
                    format: image.format.clone(),
                    width: image.width,
                    height: image.height,
                }),
                source_image_path: Some(path),
                units,
            }
        })
        .collect();

    ArchiveDocument {
        comic: ArchiveComic {
            title: snapshot.comic.title.clone(),
            subtitle: snapshot.comic.subtitle.clone(),
            author: snapshot.comic.author.clone(),
        },
        pages,
        warnings: Vec::new(),
    }
}
