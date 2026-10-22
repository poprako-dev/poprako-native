use poprako_orchestra::Run;

use crate::data::terminology::TerminologyContext;
use crate::data::terminology::portable::PortableTermbase;
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::terminology::ReadTermbaseExport;
use crate::result::AppResult;

/// # Errors
/// Reads the complete visible library from one snapshot without changing content or work state.
pub async fn export_termbase(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    id: String,
) -> AppResult<PortableTermbase> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadTermbaseExport { access, id })
        .await
}
