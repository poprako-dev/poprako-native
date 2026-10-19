use crate::data::editor::PageEditor;
use crate::model::chapter::Chapter;

#[derive(Debug, Clone)]
pub struct ExportSnapshot {
    pub comic: crate::model::comic::Comic,
    pub chapter: Chapter,
    pub pages: Vec<PageEditor>,
}
