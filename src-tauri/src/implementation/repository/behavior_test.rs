use std::error::Error;
use std::path::PathBuf;

use uuid::Uuid;

use crate::data::comic::ComicMetadata;
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::page::{NewPage, PageBaseline, RemovePages, ReorderPages};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::model::page::Page;
use crate::model::work_position::{EditorMode, WorkPosition};
use crate::result::AppError;
use crate::usecase::{comic, editor, page, preference, work_position};
use crate::value::image::{Image, ImageFormat};

fn require_send<T: Send>(value: T) -> T {
    value
}

fn image() -> NewPage {
    let id = Uuid::new_v4().to_string();

    NewPage {
        image: Image {
            reference: format!("library/{id}/original/image.png"),
            original_name: "scan.png".to_owned(),
            format: ImageFormat::Png,
            width: 1200,
            height: 1800,
        },
        id,
    }
}

fn draft(text: &str) -> UnitDraft {
    UnitDraft {
        id: Uuid::new_v4().to_string(),
        x_coord: 0.5,
        y_coord: 0.25,
        is_bubble: true,
        is_flagged: false,
        translated_text: text.to_owned(),
        proofread_text: String::new(),
        is_proofread: false,
    }
}

fn baseline(pages: &[Page]) -> Vec<PageBaseline> {
    pages
        .iter()
        .map(|page| PageBaseline {
            id: page.id.clone(),
            unit_revision: page.unit_revision,
            image_reference: page.image.reference.clone(),
        })
        .collect()
}

async fn fixture() -> Result<(SqliteCoordinator, PathBuf, String, Vec<Page>), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("poprako-native-test-{}.sqlite", Uuid::new_v4()));

    let pool = database::open(&path).await?;

    let coordinator = SqliteCoordinator::new(pool);

    let comic = require_send(comic::create_comic(
        &coordinator,
        ComicMetadata {
            title: "  漫画 \u{2003}".to_owned(),
            subtitle: "\t".to_owned(),
            author: "作者".to_owned(),
        },
    ))
    .await?;

    assert_eq!(comic.title, "漫画");

    assert_eq!(comic.subtitle, "");

    let pages =
        page::append_pages(&coordinator, &comic.id, &[], &[image(), image(), image()]).await?;

    Ok((coordinator, path, comic.id, pages))
}

async fn cleanup(coordinator: SqliteCoordinator, path: PathBuf) -> Result<(), Box<dyn Error>> {
    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}

#[tokio::test]
async fn snapshot_diff_noop_reorder_conflict_and_reopen() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, comic_id, pages) = fixture().await?;

    let page_id = pages[0].id.clone();

    let first = draft(" \n\u{2003}");

    let second = draft("  保留缩进\n");

    let request = SavePageUnits {
        comic_id: comic_id.clone(),
        page_id: page_id.clone(),
        expected_revision: 0,
        units: vec![first.clone(), second.clone()],
    };

    let saved = require_send(editor::save_page_units(&coordinator, request.clone())).await?;

    assert_eq!(saved.page.unit_revision, 1);

    assert_eq!(saved.units[0].translated_text, "");

    assert_eq!(saved.units[1].translated_text, second.translated_text);

    let noop = editor::save_page_units(
        &coordinator,
        SavePageUnits {
            expected_revision: 1,
            ..request.clone()
        },
    )
    .await?;

    assert_eq!(noop, saved);

    assert_eq!(
        editor::save_page_units(&coordinator, request).await,
        Err(AppError::Conflict)
    );

    let reordered = editor::save_page_units(
        &coordinator,
        SavePageUnits {
            comic_id: comic_id.clone(),
            page_id: page_id.clone(),
            expected_revision: 1,
            units: vec![second.clone(), first.clone()],
        },
    )
    .await?;

    assert_eq!(reordered.units[0].id, second.id);

    assert_eq!(reordered.units[0].created_at, saved.units[1].created_at);

    assert_eq!(reordered.page.unit_revision, 2);

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        editor::get_page_editor(&reopened, &comic_id, &page_id).await?,
        reordered
    );

    cleanup(reopened, path).await
}

#[tokio::test]
async fn collision_rolls_back_deletion_and_work_reference() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, comic_id, pages) = fixture().await?;

    let first = draft("first");

    let collision = draft("other page");

    for (target, unit) in [(0, first.clone()), (1, collision.clone())] {
        editor::save_page_units(
            &coordinator,
            SavePageUnits {
                comic_id: comic_id.clone(),
                page_id: pages[target].id.clone(),
                expected_revision: 0,
                units: vec![unit],
            },
        )
        .await?;
    }

    work_position::update_work_position(
        &coordinator,
        WorkPosition {
            comic_id: comic_id.clone(),
            page_id: Some(pages[0].id.clone()),
            unit_id: Some(first.id.clone()),
            mode: EditorMode::Proofreading,
            last_opened_at: 0,
        },
    )
    .await?;

    let before = editor::get_page_editor(&coordinator, &comic_id, &pages[0].id).await?;

    let result = editor::save_page_units(
        &coordinator,
        SavePageUnits {
            comic_id: comic_id.clone(),
            page_id: pages[0].id.clone(),
            expected_revision: 1,
            units: vec![collision],
        },
    )
    .await;

    assert_eq!(result, Err(AppError::Conflict));

    assert_eq!(
        editor::get_page_editor(&coordinator, &comic_id, &pages[0].id).await?,
        before
    );

    let detail = comic::get_comic_detail(&coordinator, &comic_id).await?;

    assert_eq!(
        detail.work_position.and_then(|position| position.unit_id),
        Some(first.id)
    );

    cleanup(coordinator, path).await
}

#[tokio::test]
async fn page_permutations_and_delete_restore_position() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, comic_id, pages) = fixture().await?;

    let ids: Vec<_> = pages.iter().rev().map(|page| page.id.clone()).collect();

    let reordered = page::reorder_pages(
        &coordinator,
        ReorderPages {
            comic_id: comic_id.clone(),
            baseline: baseline(&pages),
            page_ids: ids.clone(),
        },
    )
    .await?;

    assert_eq!(
        reordered.iter().map(|page| &page.id).collect::<Vec<_>>(),
        ids.iter().collect::<Vec<_>>()
    );

    assert_eq!(
        page::append_pages(&coordinator, &comic_id, &baseline(&pages), &[image()]).await,
        Err(AppError::Conflict)
    );

    work_position::update_work_position(
        &coordinator,
        WorkPosition {
            comic_id: comic_id.clone(),
            page_id: Some(ids[1].clone()),
            unit_id: None,
            mode: EditorMode::Translation,
            last_opened_at: 0,
        },
    )
    .await?;

    let remaining = page::remove_pages(
        &coordinator,
        RemovePages {
            comic_id: comic_id.clone(),
            baseline: baseline(&reordered),
            page_ids: vec![ids[1].clone()],
        },
    )
    .await?;

    assert_eq!(remaining.len(), 2);

    assert_eq!(remaining[1].index, 1);

    let detail = comic::get_comic_detail(&coordinator, &comic_id).await?;

    assert_eq!(
        detail.work_position.and_then(|position| position.page_id),
        Some(ids[0].clone())
    );

    cleanup(coordinator, path).await
}

#[tokio::test]
async fn preference_compare_and_swap_preserves_content_time() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, comic_id, _) = fixture().await?;

    let before = comic::get_comic_detail(&coordinator, &comic_id)
        .await?
        .comic;

    let original = preference::get_preference(&coordinator).await?;

    let mut changed = original.clone();

    changed.marker_opacity = 0.5;

    preference::update_preference(&coordinator, original.clone(), changed.clone()).await?;

    assert_eq!(
        preference::update_preference(&coordinator, original.clone(), original).await,
        Err(AppError::Conflict)
    );

    assert_eq!(preference::get_preference(&coordinator).await?, changed);

    assert_eq!(
        comic::get_comic_detail(&coordinator, &comic_id)
            .await?
            .comic,
        before
    );

    cleanup(coordinator, path).await
}
