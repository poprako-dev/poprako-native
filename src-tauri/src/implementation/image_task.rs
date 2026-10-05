pub mod cancel;
pub mod commit;
pub mod name;
pub mod prepare;
pub mod recovery;

use std::sync::Mutex;

use uuid::Uuid;

use crate::data::image_import::{ImageImportTarget, ImageTaskPhase, ImageTaskStatus};
use crate::implementation::image::ImagePipeline;
use crate::implementation::resource::{PreparedImage, ResourceStore};
use crate::result::{AppError, AppResult};

#[derive(Clone)]
pub struct PreparedFile {
    pub image: PreparedImage,
    pub original_name: String,
    pub handle: String,
}

#[derive(Clone)]
pub struct PublishedImage {
    pub reference: String,
    pub comic_id: Uuid,
    pub page_id: Uuid,
}

pub struct ImageTask {
    pub status: ImageTaskStatus,
    pub target: ImageImportTarget,
    pub files: Vec<PreparedFile>,
    pub published: Vec<PublishedImage>,
    pub obsolete: Vec<PublishedImage>,
    pub permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

#[derive(Default)]
pub struct ImageTaskRegistry {
    task: Mutex<Option<ImageTask>>,
}

impl ImageTaskRegistry {
    /// # Errors
    /// Only one foreground preparation or unresolved commit can own resources.
    pub fn begin(
        &self,
        target: ImageImportTarget,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> AppResult<ImageTaskStatus> {
        let mut current = self.task.lock().map_err(|_| AppError::RecoveryRequired)?;

        if current.as_ref().is_some_and(|task| {
            !matches!(
                task.status.phase,
                ImageTaskPhase::Completed | ImageTaskPhase::Cancelled | ImageTaskPhase::Failed
            )
        }) {
            return Err(AppError::Busy);
        }

        if current
            .as_ref()
            .is_some_and(|task| task.status.cleanup_pending)
        {
            return Err(AppError::Busy);
        }

        let status = ImageTaskStatus {
            task_id: Uuid::new_v4().to_string(),
            phase: ImageTaskPhase::Selecting,
            total_files: 0,
            processed_files: 0,
            processed_bytes: 0.0,
            message: String::new(),
            cleanup_pending: false,
            previews: Vec::new(),
        };

        *current = Some(ImageTask {
            status: status.clone(),
            target,
            files: Vec::new(),
            published: Vec::new(),
            obsolete: Vec::new(),
            permit: Some(permit),
        });

        Ok(status)
    }

    /// # Errors
    /// Rejects stale task IDs and poisoned registry state.
    pub fn update<T>(
        &self,
        task_id: &str,
        operation: impl FnOnce(&mut ImageTask) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut current = self.task.lock().map_err(|_| AppError::RecoveryRequired)?;

        let task = current
            .as_mut()
            .filter(|task| task.status.task_id == task_id)
            .ok_or(AppError::NotFound)?;

        operation(task)
    }

    /// # Errors
    /// A task from an earlier process or already replaced task cannot be inspected.
    pub fn status(&self, task_id: &str) -> AppResult<ImageTaskStatus> {
        self.update(task_id, |task| Ok(task.status.clone()))
    }

    /// # Errors
    /// Resolves only this process's prepared preview handles.
    pub fn preview(
        &self,
        handle: &str,
        resource: &ResourceStore,
        pipeline: &ImagePipeline,
    ) -> AppResult<Vec<u8>> {
        let current = self.task.lock().map_err(|_| AppError::RecoveryRequired)?;

        let task = current.as_ref().ok_or(AppError::NotFound)?;

        let file = task
            .files
            .iter()
            .find(|file| file.handle == handle)
            .ok_or(AppError::NotFound)?;

        let image = file.image.clone();

        drop(current);

        resource
            .prepared_preview(&image, pipeline)
            .map_err(|_| AppError::Storage)
    }

    /// # Errors
    /// Committing and uncertain tasks must finish verification before exit.
    pub fn request_stop(&self, task_id: &str) -> AppResult<ImageTaskPhase> {
        self.update(task_id, |task| {
            let previous = task.status.phase;

            match previous {
                ImageTaskPhase::Committing | ImageTaskPhase::Uncertain => {
                    return Err(AppError::Busy);
                }
                ImageTaskPhase::Preparing
                | ImageTaskPhase::Selecting
                | ImageTaskPhase::Publishing
                | ImageTaskPhase::Ready => {
                    task.status.phase = ImageTaskPhase::Stopping;
                }
                _ => {}
            }

            Ok(previous)
        })
    }

    /// # Errors
    /// Returns a recovery error if task state cannot be inspected.
    pub fn active_status(&self) -> AppResult<Option<ImageTaskStatus>> {
        Ok(self
            .task
            .lock()
            .map_err(|_| AppError::RecoveryRequired)?
            .as_ref()
            .map(|task| task.status.clone()))
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod recovery_test;

#[cfg(test)]
mod retry_test;
