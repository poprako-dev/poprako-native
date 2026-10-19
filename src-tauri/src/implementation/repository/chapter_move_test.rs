use std::error::Error;
use std::fs;

use tempfile::tempdir;
use uuid::Uuid;

use crate::complex::archive::{export as archive_export, native};
use crate::data::comic::MoveChapter;
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::page::NewPage;
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::implementation::repository::hierarchy_test::input;
use crate::implementation::resource::ResourceStore;
use crate::model::work_position::{EditorMode, WorkPosition};
use crate::result::AppError;
use crate::usecase::{chapter, comic, editor, export, page, work_position};
use crate::value::image::{Image, ImageFormat};

struct MoveFixture {
    source: crate::model::chapter::Chapter,
    target: crate::model::chapter::Chapter,
    page_id: String,
    reference: String,
    bytes: Vec<u8>,
    saved: crate::data::editor::PageEditor,
    position: WorkPosition,
}

async fn fixture(
    coordinator: &SqliteCoordinator,
    root: &std::path::Path,
) -> Result<MoveFixture, Box<dyn Error>> {
    let source = comic::create_comic(coordinator, input("来源")).await?;

    let target = comic::create_comic(coordinator, input("目标")).await?;

    let page_id = Uuid::new_v4().to_string();

    let reference = format!("library/{}/{page_id}/{}.png", source.id, Uuid::new_v4());

    let original_path = root.join(&reference);

    fs::create_dir_all(original_path.parent().ok_or("missing parent")?)?;

    image::RgbImage::new(2, 3).save(&original_path)?;

    let bytes = fs::read(&original_path)?;

    page::append_pages(
        coordinator,
        &source.id,
        &[],
        &[NewPage {
            id: page_id.clone(),
            image: Image {
                reference: reference.clone(),
                original_name: "原图.png".to_owned(),
                format: ImageFormat::Png,
                width: 2,
                height: 3,
            },
        }],
    )
    .await?;

    let unit_id = Uuid::new_v4().to_string();

    let saved = editor::save_page_units(
        coordinator,
        SavePageUnits {
            chapter_id: source.id.clone(),
            page_id: page_id.clone(),
            expected_revision: 0,
            units: vec![UnitDraft {
                id: unit_id.clone(),
                x_coord: 0.25,
                y_coord: 0.5,
                is_bubble: true,
                is_flagged: true,
                translated_text: "译文".to_owned(),
                proofread_text: "校对".to_owned(),
                is_proofread: true,
            }],
        },
    )
    .await?;

    let position = work_position::update_work_position(
        coordinator,
        WorkPosition {
            chapter_id: source.id.clone(),
            last_opened_at: 0,
            page_id: Some(page_id.clone()),
            unit_id: Some(unit_id),
            mode: EditorMode::Proofreading,
        },
    )
    .await?;

    Ok(MoveFixture {
        source,
        target,
        page_id,
        reference,
        bytes,
        saved,
        position,
    })
}

#[tokio::test]
async fn moving_preserves_images_content_and_work_position_but_exports_target_metadata()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let path = directory.path().join("database.sqlite");

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let MoveFixture {
        source,
        target,
        page_id,
        reference,
        bytes,
        saved,
        position,
    } = fixture(&coordinator, directory.path()).await?;

    let before = comic::list_comic_infos(&coordinator, 0, 50).await?;

    chapter::list_chapter_infos(&coordinator, &source.comic_id).await?;

    assert_eq!(comic::list_comic_infos(&coordinator, 0, 50).await?, before);

    let request = MoveChapter {
        chapter_id: source.id.clone(),
        source_comic_id: source.comic_id.clone(),
        target_comic_id: target.comic_id.clone(),
        source_baseline: vec![source.id.clone()],
        target_baseline: vec![target.id.clone()],
    };

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        comic::move_chapter(&coordinator, request.clone()).await,
        Err(AppError::Storage)
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let moved = comic::move_chapter(&coordinator, request.clone()).await?;

    assert_eq!(moved.comic_id, target.comic_id);

    assert_eq!(moved.index, 1);

    assert_eq!(
        comic::move_chapter(&coordinator, request).await,
        Err(AppError::Conflict)
    );

    assert!(
        chapter::list_chapter_infos(&coordinator, &source.comic_id)
            .await?
            .is_empty()
    );

    assert_eq!(
        editor::get_page_editor(&coordinator, &source.id, &page_id).await?,
        saved
    );

    assert!(
        editor::get_page_editor(&coordinator, &target.id, &page_id)
            .await
            .is_err()
    );

    assert_eq!(
        chapter::get_chapter_detail(&coordinator, &source.id)
            .await?
            .work_position,
        Some(position)
    );

    let snapshot = export::snapshot(&coordinator, &source.id).await?;

    let document = archive_export::document(&snapshot);

    assert_eq!(document.comic.title, "目标");

    assert_eq!(document.comic.author, "目标作者");

    assert_eq!(document.comic.subtitle, source.title);

    let parsed = native::parse(serde_json::from_slice(&native::encode(&document)?)?)?;

    assert_eq!(parsed.comic, document.comic);

    assert_eq!(parsed.pages[0].units, document.pages[0].units);

    let store = ResourceStore::open(directory.path())?;

    assert_eq!(
        fs::read(store.resolve(
            &reference,
            Uuid::parse_str(&source.id)?,
            Uuid::parse_str(&page_id)?
        )?)?,
        bytes
    );

    let infos = comic::list_comic_infos(&coordinator, 0, 50).await?;

    let info = infos
        .iter()
        .find(|info| info.comic.id == target.comic_id)
        .ok_or("missing target")?;

    assert_eq!(
        (
            info.chapter_count,
            info.page_count,
            info.unit_count,
            info.proofread_count
        ),
        (2, 1, 1, 1)
    );

    assert_eq!(info.latest_chapter_id.as_deref(), Some(source.id.as_str()));

    assert_eq!(info.cover_chapter_id.as_deref(), Some(source.id.as_str()));

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        editor::get_page_editor(&reopened, &source.id, &page_id).await?,
        saved
    );

    reopened.pool().close().await;

    Ok(())
}
