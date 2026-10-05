use serde::{Deserialize, Serialize};
use specta::Type;

use crate::value::image::Image;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Page {
    pub id: String,
    pub comic_id: String,
    pub index: u32,
    pub unit_revision: i64,
    pub image: Image,
    pub created_at: i64,
    pub updated_at: i64,
}
