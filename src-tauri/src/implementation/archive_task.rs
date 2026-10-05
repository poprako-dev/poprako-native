pub mod commit;
pub mod export;
pub mod picker;
pub mod prepare;

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::data::archive::ArchiveDocument;
use crate::data::archive_task::ArchiveTask;
use crate::data::import::ExistingComicImport;
use crate::data::page::PageBaseline;
use crate::implementation::archive::import::PublishedArchive;
use crate::implementation::archive::source::PreparedArchive;
use crate::result::{AppError, AppResult};

pub enum ImportArtifact {
    New(PreparedArchive),
    Existing {
        document: ArchiveDocument,
        comic_id: String,
        baseline: Vec<PageBaseline>,
        target_pages: Vec<(String, u32)>,
    },
}

pub enum UncertainArtifact {
    New(Box<PublishedArchive>),
    Existing(ExistingComicImport),
}

pub struct ArchiveRecord {
    pub permit: Option<tokio::sync::OwnedSemaphorePermit>,
    pub status: ArchiveTask,
    pub cancel: Arc<AtomicBool>,
    pub artifact: Option<ImportArtifact>,
    pub uncertain: Option<UncertainArtifact>,
}

#[derive(Default)]
pub struct ArchiveTasks {
    records: Mutex<HashMap<String, ArchiveRecord>>,
}

impl ArchiveTasks {
    /// # Errors
    /// Bounds live tasks and retains finished outcomes until acknowledged.
    pub fn create(
        &self,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> AppResult<(String, Arc<AtomicBool>)> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?;

        if records.len() >= 16 {
            return Err(AppError::Busy);
        }

        let id = Uuid::new_v4().to_string();

        let cancel = Arc::new(AtomicBool::new(false));

        records.insert(
            id.clone(),
            ArchiveRecord {
                permit: Some(permit),
                status: ArchiveTask::Preparing {
                    completed: 0,
                    total: 0,
                },
                cancel: Arc::clone(&cancel),
                artifact: None,
                uncertain: None,
            },
        );

        Ok((id, cancel))
    }

    /// # Errors
    /// Unknown or already consumed task handles cannot be read.
    pub fn get(&self, id: &str) -> AppResult<ArchiveTask> {
        let records = self
            .records
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?;

        Ok(records.get(id).ok_or(AppError::NotFound)?.status.clone())
    }

    /// # Errors
    /// Updates only process-owned records under their synchronization boundary.
    pub fn update<T>(
        &self,
        id: &str,
        operation: impl FnOnce(&mut ArchiveRecord) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?;

        operation(records.get_mut(id).ok_or(AppError::NotFound)?)
    }

    /// # Errors
    /// Cancellation never abandons a committing or uncertain transaction.
    pub fn cancel(&self, id: &str) -> AppResult<()> {
        self.update(id, |record| {
            if matches!(
                record.status,
                ArchiveTask::Committing
                    | ArchiveTask::Cleaning
                    | ArchiveTask::Failed {
                        commit_uncertain: true,
                        ..
                    }
            ) {
                return Err(AppError::Busy);
            }

            record.cancel.store(true, Ordering::Release);

            Ok(())
        })
    }

    /// # Errors
    /// Pending work and uncertain outcomes cannot be evicted.
    pub fn acknowledge(&self, id: &str) -> AppResult<()> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?;

        let record = records.get(id).ok_or(AppError::NotFound)?;

        if !matches!(
            record.status,
            ArchiveTask::Completed { .. }
                | ArchiveTask::Cancelled
                | ArchiveTask::Failed {
                    commit_uncertain: false,
                    ..
                }
        ) || record.artifact.is_some()
            || record.uncertain.is_some()
        {
            return Err(AppError::Busy);
        }

        records.remove(id);

        Ok(())
    }
}

/// # Errors
/// Reports cancellation only at boundaries before irreversible work.
pub fn check_cancel(cancel: &AtomicBool) -> AppResult<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(AppError::Busy);
    }

    Ok(())
}
