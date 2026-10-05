use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum EditorMode {
    Translation,
    Proofreading,
    Readonly,
}

impl EditorMode {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Translation => "translation",
            Self::Proofreading => "proofreading",
            Self::Readonly => "readonly",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WorkPosition {
    pub comic_id: String,
    pub last_opened_at: i64,
    pub page_id: Option<String>,
    pub unit_id: Option<String>,
    pub mode: EditorMode,
}
