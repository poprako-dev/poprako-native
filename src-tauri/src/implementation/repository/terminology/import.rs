use poprako_orchestra::Step;

use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::model::terminology::Termbase;
use crate::part::repository::terminology::ImportTermbase;
use crate::result::{AppError, AppResult};

impl Step<ImportTermbase, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ImportTermbase,
    ) -> AppResult<Termbase> {
        let base = self.step(context, &operation.base).await?;

        for term in &operation.terms {
            if term.termbase_id != base.id || term.access != operation.base.access {
                return Err(AppError::InvalidInput);
            }

            self.step(context, term).await?;
        }

        Ok(base)
    }
}
