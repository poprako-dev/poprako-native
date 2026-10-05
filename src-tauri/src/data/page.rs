use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct PageBaseline {
    pub id: String,
    pub unit_revision: i64,
    pub image_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ReorderPages {
    pub comic_id: String,
    pub baseline: Vec<PageBaseline>,
    pub page_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct RemovePages {
    pub comic_id: String,
    pub baseline: Vec<PageBaseline>,
    pub page_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NewPage {
    pub id: String,
    pub image: crate::value::image::Image,
}
