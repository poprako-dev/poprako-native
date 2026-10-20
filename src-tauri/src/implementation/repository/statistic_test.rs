use std::error::Error;

use uuid::Uuid;

use crate::data::comic::{ComicMetadata, NewComic};
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::page::NewPage;
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::usecase::{chapter, editor, page};
use crate::value::image::{Image, ImageFormat};

fn image() -> NewPage {
    let id = Uuid::new_v4().to_string();

    NewPage {
        image: Image {
            reference: format!("library/{id}/original/image.png"),
            original_name: "scan.png".to_owned(),
            format: ImageFormat::Png,
            width: 100,
            height: 100,
        },
        id,
    }
}

fn unit(translation: &str, proofreading: &str) -> UnitDraft {
    UnitDraft {
        id: Uuid::new_v4().to_string(),
        x_coord: 0.5,
        y_coord: 0.5,
        is_bubble: true,
        is_flagged: false,
        translated_text: translation.to_owned(),
        proofread_text: proofreading.to_owned(),
        is_proofread: false,
    }
}

#[tokio::test]
async fn detail_statistics_include_edits_additions_and_empty_pages() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("poprako-statistics-{}.sqlite", Uuid::new_v4()));

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let chapter = crate::usecase::comic::create_comic(
        &coordinator,
        NewComic {
            comic: ComicMetadata {
                title: "Statistics".to_owned(),
                author: String::new(),
            },
            chapter_title: "第1章".to_owned(),
        },
    )
    .await?;

    let pages = page::append_pages(&coordinator, &chapter.id, &[], &[image(), image()]).await?;

    editor::save_page_units(
        &coordinator,
        SavePageUnits {
            chapter_id: chapter.id.clone(),
            page_id: pages[0].id.clone(),
            expected_revision: 0,
            units: vec![
                unit("old", "new"),
                unit("same", "same"),
                unit(" \t\n", "addition"),
                unit("original", " \n"),
            ],
        },
    )
    .await?;

    let detail = chapter::get_chapter_detail(&coordinator, &chapter.id).await?;

    assert_eq!(detail.pages[0].unit_count, 4);

    assert_eq!(detail.pages[0].translated_count, 3);

    assert_eq!(detail.pages[0].edited_count, 1);

    assert_eq!(detail.pages[0].proofreader_append_count, 1);

    assert_eq!(detail.pages[0].proofread_count, 0);

    assert_eq!(detail.pages[1].unit_count, 0);

    assert_eq!(detail.pages[1].edited_count, 0);

    assert_eq!(detail.pages[1].proofreader_append_count, 0);

    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}
