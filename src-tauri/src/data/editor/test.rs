use crate::data::editor::{SavePageUnits, UnitDraft};

#[test]
fn marker_coordinates_survive_json_round_trip_exactly() -> Result<(), serde_json::Error> {
    for pixel in 1..997 {
        let coordinate = f64::from(pixel) / 997.0;

        let snapshot = SavePageUnits {
            chapter_id: "chapter".to_owned(),
            page_id: "page".to_owned(),
            expected_revision: 0,
            units: vec![UnitDraft {
                id: "marker".to_owned(),
                x_coord: coordinate,
                y_coord: 1.0 - coordinate,
                is_bubble: true,
                is_flagged: false,
                translated_text: String::new(),
                proofread_text: String::new(),
                is_proofread: false,
            }],
        };

        let json = serde_json::to_string(&snapshot)?;

        let received: SavePageUnits = serde_json::from_str(&json)?;

        assert_eq!(received, snapshot, "marker at pixel {pixel}: {json}");
    }

    Ok(())
}
