use std::sync::{Arc, Mutex};

use crate::model::comic::Comic;
use crate::model::page::Page;
use crate::model::unit::Unit;
use crate::model::work_position::WorkPosition;

// Orthogonal row projections, not mutually exclusive states; combinations describe actual affected rows.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct RecoveryScope {
    pub comic_id: Option<String>,
    pub comic_row: bool,
    pub comic_pages: bool,
    pub comic_units: bool,
    pub page_ids: Vec<String>,
    pub unit_page_ids: Vec<String>,
    pub unit_ids: Vec<String>,
    pub work_position: bool,
    pub preference: bool,
}

impl RecoveryScope {
    #[must_use]
    pub fn comic(comic_id: &str) -> Self {
        Self {
            comic_id: Some(comic_id.to_owned()),
            comic_row: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutationState {
    pub comic: Option<Comic>,
    pub pages: Vec<Page>,
    pub units: Vec<Unit>,
    pub work_position: Option<WorkPosition>,
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
