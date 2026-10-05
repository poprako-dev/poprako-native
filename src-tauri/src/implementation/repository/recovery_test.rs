use std::error::Error;
use std::path::PathBuf;

use uuid::Uuid;

use crate::data::comic::ComicMetadata;
use crate::data::editor::{PageEditor, SavePageUnits, UnitDraft};
use crate::data::import::{NewComicImport, PreparedImportPage};
use crate::data::page::NewPage;
use crate::implementation::coordinator::database;
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator};
use crate::model::work_position::{EditorMode, WorkPosition};
use crate::result::AppError;
use crate::usecase::{comic, editor, import, work_position};
use crate::value::image::{Image, ImageFormat};

async fn fixture() -> Result<(SqliteCoordinator, PathBuf, PageEditor), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("poprako-recovery-{}.sqlite", Uuid::new_v4()));

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let page_id = Uuid::new_v4().to_string();

    let unit_id = Uuid::new_v4().to_string();

    let imported = import::create_comic_from_import(
        &coordinator,
        NewComicImport {
            comic_id: Uuid::new_v4().to_string(),
            metadata: ComicMetadata {
                title: "Recovery".to_owned(),
                subtitle: String::new(),
                author: String::new(),
            },
            pages: vec![PreparedImportPage {
                page: NewPage {
                    id: page_id.clone(),
                    image: Image {
                        reference: "original.png".to_owned(),
                        original_name: "original.png".to_owned(),
                        format: ImageFormat::Png,
                        width: 1,
                        height: 1,
                    },
                },
                units: vec![UnitDraft {
                    id: unit_id.clone(),
                    x_coord: 0.0,
                    y_coord: 0.0,
                    is_bubble: true,
                    is_flagged: false,
                    translated_text: "before".to_owned(),
                    proofread_text: String::new(),
                    is_proofread: false,
                }],
            }],
        },
    )
    .await?;

    work_position::update_work_position(
        &coordinator,
        WorkPosition {
            comic_id: imported.id.clone(),
            page_id: Some(page_id.clone()),
            unit_id: Some(unit_id),
            mode: EditorMode::Translation,
            last_opened_at: 0,
        },
    )
    .await?;

    let before = editor::get_page_editor(&coordinator, &imported.id, &page_id).await?;

    Ok((coordinator, path, before))
}

fn clear_request(before: &PageEditor) -> SavePageUnits {
    SavePageUnits {
        comic_id: before.page.comic_id.clone(),
        page_id: before.page.id.clone(),
        expected_revision: before.page.unit_revision,
        units: Vec::new(),
    }
}

#[tokio::test]
async fn committed_receipt_loss_recovers_exact_saved_state() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, before) = fixture().await?;

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let saved = editor::save_page_units(&coordinator, clear_request(&before)).await?;

    assert!(saved.units.is_empty());

    assert_eq!(saved.page.unit_revision, 1);

    assert!(!coordinator.is_halted());

    let detail = comic::get_comic_detail(&coordinator, &before.page.comic_id).await?;

    assert_eq!(
        detail.work_position.and_then(|position| position.unit_id),
        None
    );

    assert_eq!(
        saved,
        editor::get_page_editor(&coordinator, &before.page.comic_id, &before.page.id).await?
    );

    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}

#[tokio::test]
async fn rolled_back_receipt_loss_does_not_claim_success() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, before) = fixture().await?;

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        editor::save_page_units(&coordinator, clear_request(&before)).await,
        Err(AppError::Storage)
    );

    assert!(!coordinator.is_halted());

    assert_eq!(
        editor::get_page_editor(&coordinator, &before.page.comic_id, &before.page.id).await?,
        before
    );

    assert!(
        comic::get_comic_detail(&coordinator, &before.page.comic_id)
            .await?
            .work_position
            .and_then(|position| position.unit_id)
            .is_some()
    );

    editor::save_page_units(&coordinator, clear_request(&before)).await?;

    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}

#[tokio::test]
async fn unavailable_verification_preserves_halt() -> Result<(), Box<dyn Error>> {
    let (coordinator, path, before) = fixture().await?;

    coordinator.inject_commit_fault(CommitFault::Unverifiable);

    let retained = std::sync::Arc::new(std::sync::Mutex::new(None));

    assert_eq!(
        editor::save_page_units_with_evidence(
            &coordinator,
            clear_request(&before),
            retained.clone()
        )
        .await,
        Err(AppError::CommitUncertain)
    );

    assert!(coordinator.is_halted());

    assert_eq!(
        editor::save_page_units(&coordinator, clear_request(&before)).await,
        Err(AppError::RecoveryRequired)
    );

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    let evidence = retained
        .lock()
        .map_err(|_| "poisoned test evidence")?
        .clone()
        .ok_or("missing retained evidence")?;

    reopened.halt_for_recovery();

    assert!(
        editor::verify_page_save(&reopened, &evidence)
            .await?
            .units
            .is_empty()
    );

    assert!(!reopened.is_halted());

    assert!(
        editor::get_page_editor(&reopened, &before.page.comic_id, &before.page.id)
            .await?
            .units
            .is_empty()
    );

    reopened.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}
