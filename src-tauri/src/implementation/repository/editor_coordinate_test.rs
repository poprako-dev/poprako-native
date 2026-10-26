use std::error::Error;

use crate::data::editor::{PageEditor, SavePageUnits, UnitDraft};
use crate::implementation::repository::behavior_test::{cleanup, draft, fixture};
use crate::usecase::editor;

#[tokio::test]
async fn marker_only_save_keeps_coordinates_through_ipc_database_and_receipt()
-> Result<(), Box<dyn Error>> {
    let (coordinator, path, chapter_id, pages) = fixture().await?;

    let units: Vec<_> = (1..32)
        .map(|pixel| UnitDraft {
            x_coord: f64::from(pixel) / 997.0,
            y_coord: 1.0 - f64::from(pixel) / 997.0,
            ..draft("")
        })
        .collect();

    let snapshot = SavePageUnits {
        chapter_id: chapter_id.clone(),
        page_id: pages[0].id.clone(),
        expected_revision: 0,
        units: units.clone(),
    };

    let wire = serde_json::to_vec(&snapshot)?;

    // Tauri parses the IPC body into a Value before deserializing the command.
    let payload: serde_json::Value = serde_json::from_slice(&wire)?;

    let received: SavePageUnits = serde_json::from_value(payload)?;

    let saved = editor::save_page_units(&coordinator, received).await?;

    let reply = serde_json::to_vec(&saved)?;

    let acknowledged: PageEditor = serde_json::from_slice(&reply)?;

    assert_eq!(acknowledged.page.unit_revision, 1);

    assert_eq!(
        acknowledged
            .units
            .iter()
            .map(UnitDraft::from)
            .collect::<Vec<_>>(),
        units,
        "a successful save must make the unchanged frontend draft clean"
    );

    let unchanged = editor::save_page_units(
        &coordinator,
        SavePageUnits {
            expected_revision: 1,
            ..snapshot
        },
    )
    .await?;

    assert_eq!(unchanged, saved);

    let reopened = editor::get_page_editor(&coordinator, &chapter_id, &pages[0].id).await?;

    assert_eq!(reopened, saved);

    cleanup(coordinator, path).await
}
