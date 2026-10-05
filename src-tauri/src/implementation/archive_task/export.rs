use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use uuid::Uuid;

use crate::data::archive_task::{ArchiveCompletion, ArchiveTask};
use crate::harness::Harness;
use crate::implementation::archive::destination;
use crate::implementation::archive_task::check_cancel;
use crate::result::{AppError, AppResult};
use crate::value::validation::position;

struct CancellableOriginal {
    file: std::fs::File,
    cancel: Arc<AtomicBool>,
}

impl std::io::Read for CancellableOriginal {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(std::io::Error::other("Archive export cancelled"));
        }

        std::io::Read::read(&mut self.file, buffer)
    }
}

async fn write(
    state: Arc<Harness>,
    id: String,
    comic_id: String,
    target: PathBuf,
    include_images: bool,
    overwrite: bool,
    cancel: Arc<AtomicBool>,
) -> AppResult<ArchiveCompletion> {
    let snapshot = crate::usecase::export::snapshot(state.database().await?, &comic_id).await?;

    tokio::task::spawn_blocking(move || {
        let document = crate::complex::archive::export::document(&snapshot);

        let mut originals = Vec::new();

        if include_images {
            for page in &snapshot.pages {
                check_cancel(&cancel)?;

                originals.push(
                    state
                        .resource
                        .open_original(
                            &page.page.image.reference,
                            Uuid::parse_str(&comic_id).map_err(|_| AppError::InvalidInput)?,
                            Uuid::parse_str(&page.page.id).map_err(|_| AppError::InvalidInput)?,
                        )
                        .map_err(|_| AppError::Storage)?,
                );
            }
        }

        check_cancel(&cancel)?;

        destination::export_controlled(
            &target,
            &document,
            include_images,
            overwrite,
            |index| {
                check_cancel(&cancel)?;

                state.archive_tasks.update(&id, |record| {
                    record.status = ArchiveTask::Preparing {
                        completed: position(index)?,
                        total: position(originals.len())?,
                    };

                    Ok(())
                })?;

                originals
                    .get(index)
                    .ok_or(AppError::NotFound)?
                    .file
                    .try_clone()
                    .map(|file| CancellableOriginal {
                        file,
                        cancel: Arc::clone(&cancel),
                    })
                    .map_err(|_| AppError::Storage)
            },
            || {
                state.archive_tasks.update(&id, |record| {
                    check_cancel(&record.cancel)?;

                    record.status = ArchiveTask::Committing;

                    Ok(())
                })
            },
        )?;

        Ok(ArchiveCompletion {
            comic_id,
            warnings: vec!["已导出完整 PRK 与有损兼容 LP".to_owned()],
        })
    })
    .await
    .map_err(|_| AppError::Storage)?
}

/// Holds original image leases until the completed temporary ZIP is published.
pub async fn run(
    state: Arc<Harness>,
    id: String,
    comic_id: String,
    target: PathBuf,
    include_images: bool,
    overwrite: bool,
    cancel: Arc<AtomicBool>,
) {
    let result = write(
        Arc::clone(&state),
        id.clone(),
        comic_id,
        target,
        include_images,
        overwrite,
        Arc::clone(&cancel),
    )
    .await;

    let recorded = state.archive_tasks.update(&id, |record| {
        record.status = match result {
            Ok(result) => ArchiveTask::Completed { result },
            Err(error) => match cancel.load(Ordering::Acquire) {
                true => ArchiveTask::Cancelled,
                false => ArchiveTask::Failed {
                    message: error.to_string(),
                    commit_uncertain: false,
                },
            },
        };

        record.permit = None;

        Ok(())
    });

    if recorded.is_err() {
        eprintln!("Archive export outcome could not be recorded");
    }
}
