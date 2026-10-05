use poprako_orchestra::nucl::{Nucl, NuclError};
use poprako_orchestra::{Oper, Run, Step};

use crate::data::recovery::{EvidenceHandle, RecoveryEvidence, RecoveryScope};
use crate::implementation::coordinator::{SqliteContext, SqliteCoordinator};
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::recovery::ReadMutationState;
use crate::result::{AppError, AppResult};

/// # Errors
/// Returns Storage only for an exact pre-write state; unavailable or mixed state remains uncertain.
pub async fn verify_evidence<T: Clone + Send + Sync>(
    coordinator: &SqliteCoordinator,
    evidence: &RecoveryEvidence<T>,
) -> AppResult<T> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    let _barrier = coordinator.recovery_barrier().await;

    let state = repository
        .run(&ReadMutationState {
            scope: evidence.scope.clone(),
        })
        .await
        .map_err(|_| AppError::CommitUncertain)?;

    if evidence.expected.as_ref() == Some(&state) {
        let result = evidence.result.clone().ok_or(AppError::CommitUncertain)?;

        coordinator.resume_after_verification();

        return Ok(result);
    }

    if state == evidence.before {
        coordinator.resume_after_verification();

        return Err(AppError::Storage);
    }

    Err(AppError::CommitUncertain)
}

/// # Errors
/// Preserves the before evidence even if the write step or its rollback fails.
pub async fn execute<O>(
    coordinator: &SqliteCoordinator,
    operation: O,
    scope: RecoveryScope,
    evidence: EvidenceHandle<O::Output>,
) -> AppResult<O::Output>
where
    O: Oper + Send + Sync,
    O::Output: Clone + Send + Sync,
    SqliteRepository: Step<
            O,
            SqliteContext,
            Error = AppError,
            Level = crate::implementation::coordinator::Immediate,
        >,
{
    let repository = SqliteRepository::new(coordinator.pool().clone());

    let result = coordinator
        .coord(async |context| {
            let read = ReadMutationState {
                scope: scope.clone(),
            };

            let before = <SqliteRepository as Step<ReadMutationState, SqliteContext>>::step(
                &repository,
                context,
                &read,
            )
            .await?;

            *evidence.lock().map_err(|_| AppError::RecoveryRequired)? = Some(RecoveryEvidence {
                scope,
                before,
                expected: None,
                result: None,
            });

            let output = repository.step(context, &operation).await?;

            let expected = <SqliteRepository as Step<ReadMutationState, SqliteContext>>::step(
                &repository,
                context,
                &read,
            )
            .await?;

            {
                let mut guard = evidence.lock().map_err(|_| AppError::RecoveryRequired)?;

                let retained = guard.as_mut().ok_or(AppError::RecoveryRequired)?;

                retained.expected = Some(expected);

                retained.result = Some(output.clone());
            }

            Ok::<_, AppError>(output)
        })
        .await;

    let uncertainty = match result {
        Ok(value) => return Ok(value),
        Err(NuclError::Backend(
            error @ (AppError::CommitUncertain | AppError::RecoveryRequired),
        )) if coordinator.is_halted() => error,
        Err(NuclError::Backend(error) | NuclError::Step(error)) => return Err(error),
    };

    let retained = evidence
        .lock()
        .map_err(|_| AppError::CommitUncertain)?
        .clone();

    let Some(retained) = retained else {
        return Err(uncertainty);
    };

    verify_evidence(coordinator, &retained).await
}
