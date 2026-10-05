use std::error::Error;

use uuid::Uuid;

use crate::data::comic::ComicMetadata;
use crate::data::editor::UnitDraft;
use crate::data::import::{ExistingComicImport, ImportMode, NewComicImport, PreparedImportPage};
use crate::data::page::{NewPage, PageBaseline};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::result::AppError;
use crate::usecase::{comic, editor, import};
use crate::value::image::{Image, ImageFormat};

fn unit() -> UnitDraft {
    UnitDraft {
        id: Uuid::new_v4().to_string(),
        x_coord: 0.5,
        y_coord: 0.5,
        is_bubble: true,
        is_flagged: true,
        translated_text: " \t".to_owned(),
        proofread_text: "校对".to_owned(),
        is_proofread: true,
    }
}

fn page(units: Vec<UnitDraft>) -> PreparedImportPage {
    let id = Uuid::new_v4().to_string();

    PreparedImportPage {
        page: NewPage {
            image: Image {
                reference: format!("library/{id}/original/image.png"),
                original_name: "image.png".to_owned(),
                format: ImageFormat::Png,
                width: 1,
                height: 1,
            },
            id,
        },
        units,
    }
}

fn document(pages: Vec<PreparedImportPage>) -> NewComicImport {
    NewComicImport {
        comic_id: Uuid::new_v4().to_string(),
        metadata: ComicMetadata {
            title: "Imported".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
        pages,
    }
}

#[tokio::test]
async fn import_is_atomic_and_new_initial_revisions_are_zero() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("poprako-import-{}.sqlite", Uuid::new_v4()));

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let duplicate = unit();

    let invalid = document(vec![page(vec![duplicate.clone()]), page(vec![duplicate])]);

    assert_eq!(
        import::create_comic_from_import(&coordinator, invalid).await,
        Err(AppError::Conflict)
    );

    assert!(
        comic::list_comic_infos(&coordinator, 0, 50)
            .await?
            .is_empty()
    );

    let input = document(vec![page(vec![unit()]), page(Vec::new())]);

    let created = import::create_comic_from_import(&coordinator, input.clone()).await?;

    let saved = editor::get_page_editor(&coordinator, &created.id, &input.pages[0].page.id).await?;

    assert_eq!(saved.page.unit_revision, 0);

    assert_eq!(saved.units[0].translated_text, "");

    assert_eq!(saved.units[0].created_at, created.created_at);

    assert_eq!(saved.page.created_at, created.created_at);

    let detail = comic::get_comic_detail(&coordinator, &created.id).await?;

    let baseline: Vec<_> = detail
        .pages
        .iter()
        .map(|info| PageBaseline {
            id: info.page.id.clone(),
            unit_revision: info.page.unit_revision,
            image_reference: info.page.image.reference.clone(),
        })
        .collect();

    let affected = import::apply_comic_import(
        &coordinator,
        ExistingComicImport {
            comic_id: created.id.clone(),
            baseline: baseline.clone(),
            pages: vec![Vec::new(), vec![unit()]],
            mode: ImportMode::FillEmpty,
        },
    )
    .await?;

    assert_eq!(affected, vec![input.pages[1].page.id.clone()]);

    assert_eq!(
        editor::get_page_editor(&coordinator, &created.id, &input.pages[0].page.id).await?,
        saved
    );

    assert_eq!(
        import::apply_comic_import(
            &coordinator,
            ExistingComicImport {
                comic_id: created.id.clone(),
                baseline,
                pages: vec![Vec::new(), Vec::new()],
                mode: ImportMode::ReplaceAll
            }
        )
        .await,
        Err(AppError::Conflict)
    );

    let detail = comic::get_comic_detail(&coordinator, &created.id).await?;

    let baseline: Vec<_> = detail
        .pages
        .iter()
        .map(|info| PageBaseline {
            id: info.page.id.clone(),
            unit_revision: info.page.unit_revision,
            image_reference: info.page.image.reference.clone(),
        })
        .collect();

    assert_eq!(
        import::apply_comic_import(
            &coordinator,
            ExistingComicImport {
                comic_id: created.id.clone(),
                baseline,
                pages: vec![Vec::new(), Vec::new()],
                mode: ImportMode::ReplaceAll
            }
        )
        .await?
        .len(),
        2
    );

    assert!(
        editor::get_page_editor(&coordinator, &created.id, &input.pages[0].page.id)
            .await?
            .units
            .is_empty()
    );

    coordinator.pool().close().await;

    std::fs::remove_file(path)?;

    Ok(())
}
