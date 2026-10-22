use crate::complex::terminology::import::normalize_terms;
use crate::data::recovery::{EvidenceHandle, RecoveryScope};
use crate::data::terminology::{TermbaseAppend, TermbaseAppendResult, TerminologyContext};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::part::repository::terminology::AppendTermbase;
use crate::result::AppResult;
use crate::usecase::recovery::execute;
use crate::usecase::terminology::{get_termbase, recovery_scope};

/// # Errors
/// Captures the entire target aggregate so uncertain merges are verified without replaying notes.
pub async fn append_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    termbase_id: String,
    document: TermbaseAppend,
    evidence: EvidenceHandle<TermbaseAppendResult>,
) -> AppResult<TermbaseAppendResult> {
    let terms = normalize_terms(document.terms)?;

    let base = get_termbase(coordinator, access.clone(), termbase_id.clone())
        .await?
        .termbase;

    let scope = RecoveryScope {
        termbase_terms: true,
        ..recovery_scope(&base.scope, base.id)
    };

    execute(
        coordinator,
        AppendTermbase {
            access,
            termbase_id,
            terms,
        },
        scope,
        evidence,
    )
    .await
}
