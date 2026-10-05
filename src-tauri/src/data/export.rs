use crate::data::editor::PageEditor;
use crate::model::comic::Comic;

#[derive(Debug, Clone)]
pub struct ExportSnapshot {
    pub comic: Comic,
    pub pages: Vec<PageEditor>,
}
