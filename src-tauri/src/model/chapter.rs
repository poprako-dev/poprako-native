use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Chapter {
    pub id: String,
    pub comic_id: String,
    pub title: String,
    pub index: u32,
    pub created_at: i64,
    pub updated_at: i64,
}
