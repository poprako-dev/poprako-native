use serde::{Deserialize, Serialize};
use specta::Type;

use crate::value::image::ImageFormat;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ArchiveComic {
    pub title: String,
    pub subtitle: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ArchiveUnit {
    pub index: u32,
    pub x_coord: f64,
    pub y_coord: f64,
    pub is_bubble: bool,
    pub is_flagged: bool,
    pub translated_text: String,
    pub proofread_text: String,
    pub is_proofread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ArchiveImage {
    pub path: String,
    pub original_name: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ArchivePage {
    pub index: u32,
    pub image: Option<ArchiveImage>,
    pub source_image_path: Option<String>,
    pub units: Vec<ArchiveUnit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ArchiveDocument {
    pub comic: ArchiveComic,
    pub pages: Vec<ArchivePage>,
    pub warnings: Vec<String>,
}
