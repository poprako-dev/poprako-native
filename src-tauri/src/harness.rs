use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::sync::OnceCell;

use crate::implementation::coordinator::SqliteCoordinator;
use crate::implementation::coordinator::database;
use crate::implementation::image::ImagePipeline;
use crate::implementation::resource::ResourceStore;
use crate::implementation::resource::registry::ResourceRegistry;
use crate::implementation::write::WriteLedger;
use crate::result::{AppError, AppResult};

pub struct Harness {
    directory: PathBuf,
    database: OnceCell<AppResult<SqliteCoordinator>>,
    resource_ready: OnceCell<AppResult<()>>,
    pub image: ImagePipeline,
    pub image_task: Arc<crate::implementation::image_task::ImageTaskRegistry>,
    pub archive_tasks: crate::implementation::archive_task::ArchiveTasks,
    pub resource_task: Arc<tokio::sync::Semaphore>,
    pub resource: ResourceStore,
    pub resource_registry: ResourceRegistry,
    pub allow_exit: AtomicBool,
    pub write: Arc<WriteLedger>,
    _lock: File,
}

impl Harness {
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// # Errors
    /// Fails without modifying the database if another process owns the directory.
    pub fn open(directory: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&directory).map_err(|_| AppError::Storage)?;

        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(directory.join("application.lock"))
            .map_err(|_| AppError::Storage)?;

        lock.try_lock().map_err(|_| AppError::Busy)?;

        let resource = ResourceStore::open(&directory).map_err(|_| AppError::Storage)?;

        Ok(Self {
            directory,
            database: OnceCell::new(),
            resource_ready: OnceCell::new(),
            image: ImagePipeline::default(),
            image_task: Arc::new(crate::implementation::image_task::ImageTaskRegistry::default()),
            archive_tasks: crate::implementation::archive_task::ArchiveTasks::default(),
            resource_task: Arc::new(tokio::sync::Semaphore::new(1)),
            resource,
            resource_registry: ResourceRegistry::default(),
            allow_exit: AtomicBool::new(false),
            write: Arc::new(WriteLedger::default()),
            _lock: lock,
        })
    }

    /// # Errors
    /// Resource tasks wait for migration and orphan recovery before accepting new files.
    pub async fn ensure_resources(self: &Arc<Self>) -> AppResult<()> {
        self.resource_ready
            .get_or_init(|| async {
                crate::implementation::image_task::recovery::recover(Arc::clone(self)).await
            })
            .await
            .clone()
    }

    /// # Errors
    /// Preserves a failed migration result until the application restarts.
    pub async fn database(&self) -> AppResult<&SqliteCoordinator> {
        self.database
            .get_or_init(|| async {
                database::open(&self.directory.join("library.sqlite3"))
                    .await
                    .map(SqliteCoordinator::new)
            })
            .await
            .as_ref()
            .map_err(Clone::clone)
    }
}
