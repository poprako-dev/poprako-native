use poprako_orchestra::Run;
use uuid::Uuid;

use crate::data::recovery::{EvidenceHandle, RecoveryScope};
use crate::data::terminology::{
    TermMetadata, TermbaseInfo, TermbaseMetadata, TerminologyContext, TerminologyQuery,
};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::terminology::{Term, Termbase, TermbaseScope};
use crate::part::repository::terminology::{
    CreateTerm, CreateTermbase, DeleteTerm, DeleteTermbase, ListTermInfos, ListTermbaseInfos,
    ReadTermbase, ResolveTerminologyScope, UpdateTerm, UpdateTermbase,
};
use crate::result::AppResult;
use crate::usecase::recovery::execute;

pub mod export;
pub mod import;

fn recovery_scope(scope: &TermbaseScope, base_id: String) -> RecoveryScope {
    let comic_ids = match scope {
        TermbaseScope::Global => Vec::new(),
        TermbaseScope::Comic { comic_id } => vec![comic_id.clone()],
    };

    RecoveryScope {
        comic_ids,
        termbase_ids: vec![base_id],
        ..RecoveryScope::default()
    }
}

/// # Errors
/// Reads the first ordered translation from a visible library.
pub async fn preferred_translation(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    id: String,
) -> AppResult<String> {
    let term = SqliteRepository::new(coordinator.pool().clone())
        .run(&crate::part::repository::terminology::ReadTerm { access, id })
        .await?;

    term.targets
        .into_iter()
        .next()
        .ok_or(crate::result::AppError::RecoveryRequired)
}

/// # Errors
/// Returns global libraries or the current comic's libraries together with global libraries.
pub async fn list_termbase_infos(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    query: TerminologyQuery,
) -> AppResult<Vec<TermbaseInfo>> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ListTermbaseInfos { access, query })
        .await
}

/// # Errors
/// Rejects libraries outside the explicit reading context.
pub async fn get_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    id: String,
) -> AppResult<TermbaseInfo> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadTermbase { access, id })
        .await
}

/// # Errors
/// Returns one bounded page without interpreting SQL wildcard characters.
pub async fn list_term_infos(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    termbase_id: String,
    query: TerminologyQuery,
) -> AppResult<Vec<Term>> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ListTermInfos {
            access,
            termbase_id,
            query,
        })
        .await
}

/// # Errors
/// Retains the generated identity so uncertain creation is verified without creating another library.
pub async fn create_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    metadata: TermbaseMetadata,
    evidence: EvidenceHandle<Termbase>,
) -> AppResult<Termbase> {
    let owner = SqliteRepository::new(coordinator.pool().clone())
        .run(&ResolveTerminologyScope {
            access: access.clone(),
        })
        .await?;

    let id = Uuid::new_v4().to_string();

    let scope = recovery_scope(&owner, id.clone());

    execute(
        coordinator,
        CreateTermbase {
            access,
            scope: owner,
            id,
            metadata,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Rejects stale metadata and verifies both library and parent content time.
pub async fn update_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    baseline: Termbase,
    metadata: TermbaseMetadata,
    evidence: EvidenceHandle<Termbase>,
) -> AppResult<Termbase> {
    let base = get_termbase(coordinator, access.clone(), baseline.id.clone())
        .await?
        .termbase;

    let scope = recovery_scope(&base.scope, base.id);

    execute(
        coordinator,
        UpdateTermbase {
            access,
            baseline,
            metadata,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Includes all cascaded terms in the exact before/after evidence.
pub async fn delete_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    baseline: Termbase,
    evidence: EvidenceHandle<()>,
) -> AppResult<()> {
    let base = get_termbase(coordinator, access.clone(), baseline.id.clone())
        .await?
        .termbase;

    let scope = RecoveryScope {
        termbase_terms: true,
        ..recovery_scope(&base.scope, base.id)
    };

    execute(
        coordinator,
        DeleteTermbase { access, baseline },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Checks the parent's scope and records the final term identity before writing.
pub async fn create_term(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    termbase_id: String,
    metadata: TermMetadata,
    evidence: EvidenceHandle<Term>,
) -> AppResult<Term> {
    let base = get_termbase(coordinator, access.clone(), termbase_id.clone())
        .await?
        .termbase;

    let id = Uuid::new_v4().to_string();

    let scope = RecoveryScope {
        term_ids: vec![id.clone()],
        ..recovery_scope(&base.scope, base.id)
    };

    execute(
        coordinator,
        CreateTerm {
            access,
            termbase_id,
            id,
            metadata,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Preserves ordering of translations and refuses stale edits.
pub async fn update_term(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    baseline: Term,
    metadata: TermMetadata,
    evidence: EvidenceHandle<Term>,
) -> AppResult<Term> {
    let base = get_termbase(coordinator, access.clone(), baseline.termbase_id.clone())
        .await?
        .termbase;

    let scope = RecoveryScope {
        term_ids: vec![baseline.id.clone()],
        ..recovery_scope(&base.scope, base.id)
    };

    execute(
        coordinator,
        UpdateTerm {
            access,
            baseline,
            metadata,
        },
        scope,
        evidence,
    )
    .await
}

/// # Errors
/// Deletes only the exact term that was confirmed by the user.
pub async fn delete_term(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    baseline: Term,
    evidence: EvidenceHandle<()>,
) -> AppResult<()> {
    let base = get_termbase(coordinator, access.clone(), baseline.termbase_id.clone())
        .await?
        .termbase;

    let scope = RecoveryScope {
        term_ids: vec![baseline.id.clone()],
        ..recovery_scope(&base.scope, base.id)
    };

    execute(
        coordinator,
        DeleteTerm { access, baseline },
        scope,
        evidence,
    )
    .await
}
