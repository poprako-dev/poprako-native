use poprako_orchestra::Run;

use crate::data::export::ExportSnapshot;
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::export::ReadExportSnapshot;
use crate::result::AppResult;

/// Reads all exchange content from a single `SQLite` read snapshot.
/// The caller must retain the immutable original-image references while exporting.
/// # Errors
/// Reports invalid or missing comic identity and storage failures.
pub async fn snapshot(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
) -> AppResult<ExportSnapshot> {
    SqliteRepository::new(coordinator.pool().clone())
        .run(&ReadExportSnapshot {
            comic_id: comic_id.to_owned(),
        })
        .await
}
