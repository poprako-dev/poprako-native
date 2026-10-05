use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Unit {
    pub id: String,
    pub page_id: String,
    pub index: u32,
    pub x_coord: f64,
    pub y_coord: f64,
    pub is_bubble: bool,
    pub is_flagged: bool,
    pub translated_text: String,
    pub proofread_text: String,
    pub is_proofread: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
