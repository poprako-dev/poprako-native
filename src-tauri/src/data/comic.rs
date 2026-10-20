use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::comic::Comic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ComicMetadata {
    pub title: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct NewComic {
    pub comic: ComicMetadata,
    pub chapter_title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ComicInfo {
    pub comic: Comic,
    pub chapter_count: u32,
    pub page_count: u32,
    pub unit_count: u32,
    pub translated_count: u32,
    pub proofread_count: u32,
    pub cover_page_id: Option<String>,
    pub cover_chapter_id: Option<String>,
    pub latest_chapter_id: Option<String>,
    pub last_opened_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ChapterOrder {
    pub comic_id: String,
    pub baseline: Vec<String>,
    pub ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct MoveChapter {
    pub chapter_id: String,
    pub source_comic_id: String,
    pub target_comic_id: String,
    pub source_baseline: Vec<String>,
    pub target_baseline: Vec<String>,
}
