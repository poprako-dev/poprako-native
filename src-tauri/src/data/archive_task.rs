use serde::{Deserialize, Serialize};
use specta::Type;

use crate::data::comic::ComicMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArchiveTarget {
    New,
    Existing { comic_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveImportMode {
    FillEmpty,
    ReplaceAll,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ArchiveSelection {
    Cancelled,
    Selected { task_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchivePagePreview {
    pub index: u32,
    pub source_image_name: String,
    pub target_image_name: String,
    pub source_unit_count: u32,
    pub target_unit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchivePreview {
    pub pages: Vec<ArchivePagePreview>,
    pub metadata: ComicMetadata,
    pub page_count: u32,
    pub unit_count: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveCompletion {
    pub comic_id: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum ArchiveTask {
    Preparing {
        completed: u32,
        total: u32,
    },
    AwaitingConfirmation {
        preview: ArchivePreview,
    },
    Committing,
    Cleaning,
    Completed {
        result: ArchiveCompletion,
    },
    Failed {
        message: String,
        commit_uncertain: bool,
    },
    Cancelled,
}
