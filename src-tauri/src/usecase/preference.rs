use poprako_orchestra::nucl::{Nucl, NuclError};
use poprako_orchestra::{Run, Step};

use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::preference::ApplicationPreference;
use crate::part::repository::operation::{PersistPreference, ReadPreference};
use crate::result::AppResult;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn get_preference(coordinator: &SqliteCoordinator) -> AppResult<ApplicationPreference> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadPreference)
        .await
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn update_preference(
    coordinator: &SqliteCoordinator,
    baseline: ApplicationPreference,
    preference: ApplicationPreference,
) -> AppResult<ApplicationPreference> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    let operation = PersistPreference {
        baseline,
        preference,
    };

    match coordinator
        .coord(async |context| repository.step(context, &operation).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}
