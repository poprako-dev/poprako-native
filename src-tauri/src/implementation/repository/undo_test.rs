use std::error::Error;

use crate::data::editor::SavePageUnits;
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::implementation::repository::behavior_test::{cleanup, draft, fixture};
use crate::usecase::editor;

#[tokio::test]
async fn saved_deletion_restores_original_identity_and_unsent_content_after_reopen()
-> Result<(), Box<dyn Error>> {
    let (coordinator, path, chapter_id, pages) = fixture().await?;

    let page_id = pages[0].id.clone();

    let first = draft("first");

    let second = draft("second");

    let mut request = SavePageUnits {
        chapter_id: chapter_id.clone(),
        page_id: page_id.clone(),
        expected_revision: 0,
        units: vec![first.clone(), second.clone()],
    };

    editor::save_page_units(&coordinator, request.clone()).await?;

    request.expected_revision = 1;

    request.units = vec![second.clone()];

    let deleted = editor::save_page_units(&coordinator, request.clone()).await?;

    assert_eq!(deleted.page.unit_revision, 2);

    let mut restored = first.clone();

    restored.translated_text = "edited before deletion, never saved".to_owned();

    restored.proofread_text = "校对".to_owned();

    restored.is_proofread = true;

    restored.is_flagged = true;

    restored.is_bubble = false;

    restored.x_coord = 0.9;

    request.expected_revision = 2;

    request.units = vec![restored.clone(), second];

    let saved = editor::save_page_units(&coordinator, request.clone()).await?;

    assert_eq!(saved.page.unit_revision, 3);

    assert_eq!(saved.units[0].id, first.id);

    assert_eq!(saved.units[0].index, 0);

    assert_eq!(saved.units[1].index, 1);

    assert_eq!(
        crate::data::editor::UnitDraft::from(&saved.units[0]),
        restored
    );

    request.expected_revision = 3;

    assert_eq!(editor::save_page_units(&coordinator, request).await?, saved);

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        editor::get_page_editor(&reopened, &chapter_id, &page_id).await?,
        saved
    );

    cleanup(reopened, path).await
}
