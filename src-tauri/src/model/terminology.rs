use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TermbaseScope {
    Global,
    Comic { comic_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Termbase {
    pub id: String,
    pub scope: TermbaseScope,
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Term {
    pub id: String,
    pub termbase_id: String,
    pub source: String,
    pub targets: Vec<String>,
    pub comment: String,
    pub created_at: i64,
    pub updated_at: i64,
}
