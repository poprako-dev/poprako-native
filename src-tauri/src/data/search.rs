use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TextStage {
    Translation,
    Proofreading,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct SearchUnits {
    pub comic_id: String,
    pub stage: TextStage,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SearchHit {
    pub unit_id: String,
    pub page_id: String,
    pub page_index: u32,
    pub unit_index: u32,
    pub unit_revision: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub origin: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct UnitReplacement {
    pub hit: SearchHit,
    pub rules: Vec<Replacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct ReplaceUnits {
    pub comic_id: String,
    pub stage: TextStage,
    pub units: Vec<UnitReplacement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ReplacementResult {
    pub changed_unit_count: u32,
    pub changed_page_ids: Vec<String>,
}
