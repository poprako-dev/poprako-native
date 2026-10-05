use poprako_orchestra::{Run, Step};

use crate::data::recovery::MutationState;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::part::repository::recovery::ReadMutationState;
use crate::result::{AppError, AppResult};

pub mod state;

impl Step<ReadMutationState, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ReadMutationState,
    ) -> AppResult<MutationState> {
        state::capture(&mut context.0, &operation.scope).await
    }
}

impl Run<ReadMutationState> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadMutationState) -> AppResult<MutationState> {
        // The caller holds the process writer barrier; IMMEDIATE also waits for SQLite's writer.
        let mut transaction = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        let state = state::capture(&mut transaction, &operation.scope).await?;

        transaction.rollback().await?;

        Ok(state)
    }
}
