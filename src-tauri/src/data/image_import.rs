use serde::{Deserialize, Serialize};
use specta::Type;

use crate::data::page::PageBaseline;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageImportTarget {
    Append {
        comic_id: String,
        baseline: Vec<PageBaseline>,
    },
    Replace {
        comic_id: String,
        baseline: PageBaseline,
    },
}

impl ImageImportTarget {
    #[must_use]
    pub fn comic_id(&self) -> &str {
        match self {
            Self::Append { comic_id, .. } | Self::Replace { comic_id, .. } => comic_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ImageTaskPhase {
    Selecting,
    Preparing,
    Stopping,
    Ready,
    Publishing,
    Committing,
    Completed,
    Cancelled,
    Failed,
    Uncertain,
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct ImageImportPreview {
    pub handle: String,
    pub original_name: String,
    pub width: u32,
    pub height: u32,
    pub byte_length: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct ImageTaskStatus {
    pub task_id: String,
    pub phase: ImageTaskPhase,
    pub total_files: u32,
    pub processed_files: u32,
    pub processed_bytes: f64,
    pub message: String,
    pub cleanup_pending: bool,
    pub previews: Vec<ImageImportPreview>,
}
