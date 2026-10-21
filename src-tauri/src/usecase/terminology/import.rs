use poprako_orchestra::Run;
use uuid::Uuid;

use crate::complex::terminology::import::normalize;
use crate::data::recovery::{EvidenceHandle, RecoveryScope};
use crate::data::terminology::{TermbaseImport, TerminologyContext};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::terminology::{Termbase, TermbaseScope};
use crate::part::repository::terminology::{
    CreateTerm, CreateTermbase, ImportTermbase, ResolveTerminologyScope,
};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

/// # Errors
/// Creates a complete library atomically and retains all imported terms in commit evidence.
pub async fn import_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    document: TermbaseImport,
    evidence: EvidenceHandle<Termbase>,
) -> AppResult<Termbase> {
    let document = normalize(document)?;

    let owner = SqliteRepository::new(coordinator.pool().clone())
        .run(&ResolveTerminologyScope {
            access: access.clone(),
        })
        .await?;

    let comic_ids = match &owner {
        TermbaseScope::Global => Vec::new(),
        TermbaseScope::Comic { comic_id } => vec![comic_id.clone()],
    };

    let id = Uuid::new_v4().to_string();

    let scope = RecoveryScope {
        comic_ids,
        termbase_ids: vec![id.clone()],
        termbase_terms: true,
        ..RecoveryScope::default()
    };

    let terms = document
        .terms
        .into_iter()
        .map(|metadata| CreateTerm {
            access: access.clone(),
            termbase_id: id.clone(),
            id: Uuid::new_v4().to_string(),
            metadata,
        })
        .collect();

    execute(
        coordinator,
        ImportTermbase {
            base: CreateTermbase {
                access,
                scope: owner,
                id,
                metadata: document.metadata,
            },
            terms,
        },
        scope,
        evidence,
    )
    .await
}
