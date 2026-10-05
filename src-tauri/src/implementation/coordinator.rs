use std::sync::atomic::{AtomicBool, Ordering};

use poprako_orchestra::nucl::{Nucl, NuclError};
use poprako_orchestra::{Context, Level};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::result::AppError;

pub mod database;
pub mod preference_migration;
pub mod shortcut_migration;

pub struct Immediate;

impl Level for Immediate {}

pub struct SqliteContext(pub Transaction<'static, Sqlite>, pub i64);

impl Context for SqliteContext {
    type Level = Immediate;
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub enum CommitFault {
    LostCommitted,
    LostRolledBack,
    Unverifiable,
}

pub struct SqliteCoordinator {
    pool: SqlitePool,
    write: Mutex<()>,
    halted: AtomicBool,
    #[cfg(test)]
    fault: std::sync::atomic::AtomicU8,
}

impl SqliteCoordinator {
    #[cfg(test)]
    pub fn inject_commit_fault(&self, fault: CommitFault) {
        let value = match fault {
            CommitFault::LostCommitted => 1,
            CommitFault::LostRolledBack => 2,
            CommitFault::Unverifiable => 3,
        };

        self.fault.store(value, Ordering::Release);
    }

    pub async fn recovery_barrier(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.write.lock().await
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.halted.load(Ordering::Acquire)
    }

    pub fn halt_for_recovery(&self) {
        self.halted.store(true, Ordering::Release);
    }

    /// The caller must first verify the pending transaction through the write receipt ledger.
    /// This does not retry or reverse any committed transaction.
    pub fn resume_after_verification(&self) {
        self.halted.store(false, Ordering::Release);
    }

    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            write: Mutex::new(()),
            halted: AtomicBool::new(false),
            #[cfg(test)]
            fault: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Nucl for SqliteCoordinator {
    type Level = Immediate;
    type Error = AppError;
    type Context = SqliteContext;

    async fn coord<F, T, E>(&self, operation: F) -> Result<T, NuclError<Self::Error, E>>
    where
        F: for<'context> AsyncFnOnce(&'context mut Self::Context) -> Result<T, E> + Send,
        T: Send,
        E: Send,
    {
        let _guard = self.write.lock().await;

        if self.is_halted() {
            return Err(NuclError::Backend(AppError::RecoveryRequired));
        }

        let transaction = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(AppError::from)
            .map_err(NuclError::Backend)?;

        let now = crate::value::validation::timestamp().map_err(NuclError::Backend)?;

        let mut context = SqliteContext(transaction, now);

        match operation(&mut context).await {
            Ok(value) => {
                #[cfg(test)]
                let fault = self.fault.swap(0, Ordering::AcqRel);

                #[cfg(test)]
                if fault == 2 {
                    context
                        .0
                        .rollback()
                        .await
                        .map_err(AppError::from)
                        .map_err(NuclError::Backend)?;

                    self.halt_for_recovery();

                    return Err(NuclError::Backend(AppError::CommitUncertain));
                }

                context.0.commit().await.map_err(|_| {
                    self.halt_for_recovery();

                    NuclError::Backend(AppError::CommitUncertain)
                })?;

                #[cfg(test)]
                if fault == 3 {
                    self.pool.close().await;
                }

                #[cfg(test)]
                if fault == 1 || fault == 3 {
                    self.halt_for_recovery();

                    return Err(NuclError::Backend(AppError::CommitUncertain));
                }

                Ok(value)
            }
            Err(error) => {
                context.0.rollback().await.map_err(|_| {
                    self.halt_for_recovery();

                    NuclError::Backend(AppError::RecoveryRequired)
                })?;

                Err(NuclError::Step(error))
            }
        }
    }
}

#[cfg(test)]
mod database_test;

#[cfg(test)]
mod preference_migration_test;

#[cfg(test)]
mod shortcut_migration_test;
