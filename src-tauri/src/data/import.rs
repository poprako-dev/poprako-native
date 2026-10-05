use crate::data::comic::ComicMetadata;
use crate::data::editor::UnitDraft;
use crate::data::page::{NewPage, PageBaseline};

#[derive(Debug, Clone)]
pub struct PreparedImportPage {
    pub page: NewPage,
    pub units: Vec<UnitDraft>,
}

#[derive(Debug, Clone)]
pub struct NewComicImport {
    pub comic_id: String,
    pub metadata: ComicMetadata,
    pub pages: Vec<PreparedImportPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportMode {
    FillEmpty,
    ReplaceAll,
}

#[derive(Debug, Clone)]
pub struct ExistingComicImport {
    pub comic_id: String,
    pub baseline: Vec<PageBaseline>,
    pub pages: Vec<Vec<UnitDraft>>,
    pub mode: ImportMode,
}
