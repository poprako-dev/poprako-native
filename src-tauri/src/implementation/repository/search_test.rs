use std::error::Error;

use uuid::Uuid;

use crate::data::comic::ComicMetadata;
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::page::NewPage;
use crate::data::search::{ReplaceUnits, Replacement, SearchUnits, TextStage, UnitReplacement};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::result::AppError;
use crate::usecase::{comic, editor, page, search};
use crate::value::image::{Image, ImageFormat};

async fn fixture() -> Result<
    (
        SqliteCoordinator,
        std::path::PathBuf,
        crate::model::comic::Comic,
        Vec<crate::model::page::Page>,
        Vec<UnitDraft>,
    ),
    Box<dyn Error>,
> {
    let path = std::env::temp_dir().join(format!("poprako-search-{}.sqlite", Uuid::new_v4()));

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let comic = comic::create_comic(
        &coordinator,
        ComicMetadata {
            title: "Test".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
    )
    .await?;

    let pages = page::append_pages(
        &coordinator,
        &comic.id,
        &[],
        &[NewPage {
            id: Uuid::new_v4().to_string(),
            image: Image {
                reference: "test/original.png".to_owned(),
                original_name: "original.png".to_owned(),
                format: ImageFormat::Png,
                width: 1,
                height: 1,
            },
        }],
    )
    .await?;

    let units: Vec<_> = (0..101)
        .map(|_| UnitDraft {
            id: Uuid::new_v4().to_string(),
            x_coord: 0.0,
            y_coord: 0.0,
            is_bubble: true,
            is_flagged: false,
            translated_text: "I love you".to_owned(),
            proofread_text: "校对".to_owned(),
            is_proofread: true,
        })
        .collect();

    editor::save_page_units(
        &coordinator,
        SavePageUnits {
            comic_id: comic.id.clone(),
            page_id: pages[0].id.clone(),
            expected_revision: 0,
            units: units.clone(),
        },
    )
    .await?;

    Ok((coordinator, path, comic, pages, units))
}

#[tokio::test]
async fn search_limit_stage_case_and_atomic_replacement() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, comic, pages, units) = fixture().await?;

    let query = SearchUnits {
        comic_id: comic.id.clone(),
        stage: TextStage::Translation,
        query: " love ".to_owned(),
    };

    assert_eq!(
        search::search_units(&coordinator, query.clone()).await,
        Err(AppError::InvalidInput)
    );

    assert!(
        search::search_units(
            &coordinator,
            SearchUnits {
                query: "LOVE".to_owned(),
                ..query.clone()
            }
        )
        .await?
        .is_empty()
    );

    assert!(
        search::search_units(
            &coordinator,
            SearchUnits {
                query: "love\0".to_owned(),
                ..query.clone()
            }
        )
        .await?
        .is_empty()
    );

    editor::save_page_units(
        &coordinator,
        SavePageUnits {
            comic_id: comic.id.clone(),
            page_id: pages[0].id.clone(),
            expected_revision: 1,
            units: units[..2].to_vec(),
        },
    )
    .await?;

    let hits = search::search_units(&coordinator, query.clone()).await?;

    let valid = UnitReplacement {
        hit: hits[0].clone(),
        rules: vec![Replacement {
            origin: "love".to_owned(),
            target: "like".to_owned(),
        }],
    };

    let mut invalid = valid.clone();

    invalid.hit = hits[1].clone();

    invalid.hit.text = "stale".to_owned();

    assert_eq!(
        search::replace_units(
            &coordinator,
            ReplaceUnits {
                comic_id: comic.id.clone(),
                stage: TextStage::Translation,
                units: vec![valid.clone(), invalid]
            }
        )
        .await,
        Err(AppError::Conflict)
    );

    assert_eq!(
        search::search_units(&coordinator, query.clone()).await?,
        hits
    );

    let result = search::replace_units(
        &coordinator,
        ReplaceUnits {
            comic_id: comic.id.clone(),
            stage: TextStage::Translation,
            units: vec![valid],
        },
    )
    .await?;

    assert_eq!(result.changed_unit_count, 1);

    let saved = editor::get_page_editor(&coordinator, &comic.id, &pages[0].id).await?;

    assert_eq!(saved.units[0].translated_text, "I like you");

    assert!(saved.units[0].is_proofread);

    assert_eq!(saved.units[0].proofread_text, "校对");

    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}
