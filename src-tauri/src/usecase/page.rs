use poprako_orchestra::Step;
use poprako_orchestra::nucl::{Nucl, NuclError};

use crate::data::page::{NewPage, PageBaseline, RemovePages, ReorderPages};
use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::repository::SqliteRepository;
use crate::model::page::Page;
use crate::part::repository::page::{AppendPages, DeletePages, PersistPageOrder, ReplacePageImage};
use crate::result::AppResult;
use crate::value::image::Image;

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn append_pages(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    baseline: &[PageBaseline],
    pages: &[NewPage],
) -> AppResult<Vec<Page>> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    let operation = AppendPages {
        comic_id: comic_id.to_owned(),
        baseline: baseline.to_vec(),
        pages: pages.to_vec(),
    };

    match coordinator
        .coord(async |context| repository.step(context, &operation).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn reorder_pages(
    coordinator: &SqliteCoordinator,
    input: ReorderPages,
) -> AppResult<Vec<Page>> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    match coordinator
        .coord(async |context| repository.step(context, &PersistPageOrder { input }).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn remove_pages(
    coordinator: &SqliteCoordinator,
    input: RemovePages,
) -> AppResult<Vec<Page>> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    match coordinator
        .coord(async |context| repository.step(context, &DeletePages { input }).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn replace_page_image(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    baseline: PageBaseline,
    image: Image,
    clear_units: bool,
) -> AppResult<Page> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    let operation = ReplacePageImage {
        comic_id: comic_id.to_owned(),
        baseline,
        image,
        clear_units,
    };

    match coordinator
        .coord(async |context| repository.step(context, &operation).await)
        .await
    {
        Ok(value) => Ok(value),
        Err(NuclError::Backend(error) | NuclError::Step(error)) => Err(error),
    }
}

/// # Errors
/// Returns validation, missing-content, conflict, or storage errors without changing partial state.
pub async fn get_page(
    coordinator: &SqliteCoordinator,
    comic_id: &str,
    page_id: &str,
) -> AppResult<Page> {
    let repository = SqliteRepository::new(coordinator.pool().clone());

    poprako_orchestra::Run::run(
        &repository,
        &crate::part::repository::page::ReadPage {
            comic_id: comic_id.to_owned(),
            page_id: page_id.to_owned(),
        },
    )
    .await
}
