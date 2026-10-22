use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::terminology::Termbase;

pub mod portable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TermbaseExportOutcome {
    Saved,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TerminologyContext {
    Global,
    Chapter { chapter_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct TermbaseMetadata {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct TermMetadata {
    pub source: String,
    pub targets: Vec<String>,
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct TermbaseImport {
    pub metadata: TermbaseMetadata,
    pub terms: Vec<TermMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct TermbaseAppend {
    pub terms: Vec<TermMetadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct TermbaseAppendResult {
    pub added: u32,
    pub updated: u32,
    pub unchanged: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct TermbaseInfo {
    pub termbase: Termbase,
    pub term_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct TerminologyQuery {
    pub text: String,
    pub offset: u32,
    pub limit: u32,
}
