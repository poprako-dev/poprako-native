use poprako_orchestra::nucl::{Nucl, NuclError};
use poprako_orchestra::{Run, Step};

use crate::data::search::{ReplaceUnits, ReplacementResult, SearchHit, SearchUnits};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::search::{Replace, Search};
use crate::result::AppResult;

/// # Errors
/// Rejects empty queries or more than 100 matching units; reports missing projects and storage failures.
pub async fn search_units(
    coordinator: &SqliteCoordinator,
    input: SearchUnits,
) -> AppResult<Vec<SearchHit>> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&Search { input })
        .await
}

/// # Errors
/// Rejects stale search baselines, duplicate selections, overlapping rules, or invalid limits atomically.
pub async fn replace_units(
    coordinator: &SqliteCoordinator,
    input: ReplaceUnits,
) -> AppResult<ReplacementResult> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    match coordinator
        .coord(async |context| repository.step(context, &Replace { input }).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}
