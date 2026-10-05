use poprako_orchestra::Step;
use poprako_orchestra::nucl::{Nucl, NuclError};

use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::work_position::WorkPosition;
use crate::part::repository::operation::PersistWorkPosition;
use crate::result::AppResult;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn update_work_position(
    coordinator: &SqliteCoordinator,
    position: WorkPosition,
) -> AppResult<WorkPosition> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    match coordinator
        .coord(async |context| {
            repository
                .step(context, &PersistWorkPosition { position })
                .await
        })
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}
