use std::sync::{Arc, Mutex};

use crate::model::chapter::Chapter;
use crate::model::page::Page;
use crate::model::unit::Unit;
use crate::model::work_position::WorkPosition;

// Orthogonal row projections, not mutually exclusive states; combinations describe actual affected rows.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct RecoveryScope {
    pub comic_ids: Vec<String>,
    pub chapter_id: Option<String>,
    pub chapter_row: bool,
    pub chapter_pages: bool,
    pub chapter_units: bool,
    pub page_ids: Vec<String>,
    pub unit_page_ids: Vec<String>,
    pub unit_ids: Vec<String>,
    pub work_position: bool,
    pub preference: bool,
    pub termbase_ids: Vec<String>,
    pub term_ids: Vec<String>,
    pub termbase_terms: bool,
    pub comic_terminology: bool,
}

impl RecoveryScope {
    #[must_use]
    pub fn chapter(chapter_id: &str) -> Self {
        Self {
            chapter_id: Some(chapter_id.to_owned()),
            chapter_row: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutationState {
    pub termbases: Vec<crate::model::terminology::Termbase>,
    pub terms: Vec<crate::model::terminology::Term>,
    pub comics: Vec<crate::model::comic::Comic>,
    pub chapters: Vec<Chapter>,
    pub pages: Vec<Page>,
    pub units: Vec<Unit>,
    pub work_positions: Vec<WorkPosition>,
    pub preference_payload: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecoveryEvidence<T> {
    pub scope: RecoveryScope,
    pub before: MutationState,
    pub expected: Option<MutationState>,
    pub result: Option<T>,
}

pub type EvidenceHandle<T> = Arc<Mutex<Option<RecoveryEvidence<T>>>>;

#[must_use]
pub fn evidence_handle<T>() -> EvidenceHandle<T> {
    Arc::new(Mutex::new(None))
}
