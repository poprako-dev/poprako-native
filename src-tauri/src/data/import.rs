use crate::data::archive::ArchiveComic;
use crate::data::editor::UnitDraft;
use crate::data::page::{NewPage, PageBaseline};

#[derive(Debug, Clone)]
pub struct PreparedImportPage {
    pub page: NewPage,
    pub units: Vec<UnitDraft>,
}

#[derive(Debug, Clone)]
pub enum ImportParent {
    New {
        comic_id: String,
    },
    Existing {
        comic_id: String,
        baseline: Vec<String>,
    },
}

impl ImportParent {
    #[must_use]
    pub fn comic_id(&self) -> &str {
        match self {
            Self::New { comic_id } | Self::Existing { comic_id, .. } => comic_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewChapterImport {
    pub parent: ImportParent,
    pub chapter_id: String,
    pub metadata: ArchiveComic,
    pub pages: Vec<PreparedImportPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportMode {
    FillEmpty,
    ReplaceAll,
}

#[derive(Debug, Clone)]
pub struct ExistingChapterImport {
    pub chapter_id: String,
    pub baseline: Vec<PageBaseline>,
    pub pages: Vec<Vec<UnitDraft>>,
    pub mode: ImportMode,
}
