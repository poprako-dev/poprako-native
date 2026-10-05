use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::comic::Comic;
use crate::model::page::Page;
use crate::model::work_position::WorkPosition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ComicMetadata {
    pub title: String,
    pub subtitle: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct PageInfo {
    pub page: Page,
    pub unit_count: u32,
    pub translated_count: u32,
    pub proofread_count: u32,
    pub flagged_count: u32,
    pub edited_count: u32,
    pub proofreader_append_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ComicInfo {
    pub comic: Comic,
    pub page_count: u32,
    pub unit_count: u32,
    pub translated_count: u32,
    pub proofread_count: u32,
    pub cover_page_id: Option<String>,
    pub last_opened_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ComicDetail {
    pub comic: Comic,
    pub pages: Vec<PageInfo>,
    pub work_position: Option<WorkPosition>,
}
