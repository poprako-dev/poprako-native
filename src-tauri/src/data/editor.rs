use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::page::Page;
use crate::model::unit::Unit;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct UnitDraft {
    pub id: String,
    pub x_coord: f64,
    pub y_coord: f64,
    pub is_bubble: bool,
    pub is_flagged: bool,
    pub translated_text: String,
    pub proofread_text: String,
    pub is_proofread: bool,
}

impl From<&Unit> for UnitDraft {
    fn from(unit: &Unit) -> Self {
        Self {
            id: unit.id.clone(),
            x_coord: unit.x_coord,
            y_coord: unit.y_coord,
            is_bubble: unit.is_bubble,
            is_flagged: unit.is_flagged,
            translated_text: unit.translated_text.clone(),
            proofread_text: unit.proofread_text.clone(),
            is_proofread: unit.is_proofread,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct SavePageUnits {
    pub comic_id: String,
    pub page_id: String,
    pub expected_revision: i64,
    pub units: Vec<UnitDraft>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PageEditor {
    pub page: Page,
    pub units: Vec<Unit>,
}
