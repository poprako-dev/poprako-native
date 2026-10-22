use poprako_orchestra::Run;

use crate::complex::terminology::{key, validate_query};
use crate::data::terminology::TermbaseInfo;
use crate::implementation::repository::SqliteRepository;
use crate::model::terminology::{Term, TermbaseScope};
use crate::part::repository::terminology::{ListTermInfos, ListTermbaseInfos, ReadTermbase};
use crate::result::{AppError, AppResult};

pub mod export;
pub mod import;
pub mod read;
pub mod recovery;

pub mod term;
pub mod termbase;
#[cfg(test)]
mod test;

impl Run<crate::part::repository::terminology::ReadTerm> for SqliteRepository {
    type Error = AppError;

    async fn run(
        &self,
        operation: &crate::part::repository::terminology::ReadTerm,
    ) -> AppResult<Term> {
        let mut transaction = self.pool.begin().await?;

        let term = read::term(&mut transaction, &operation.id).await?;

        read::accessible_base(
            &mut transaction,
            &operation.access,
            &term.termbase_id,
            false,
        )
        .await?;

        transaction.rollback().await?;

        Ok(term)
    }
}

impl Run<crate::part::repository::terminology::ResolveTerminologyScope> for SqliteRepository {
    type Error = AppError;

    async fn run(
        &self,
        operation: &crate::part::repository::terminology::ResolveTerminologyScope,
    ) -> AppResult<TermbaseScope> {
        read::resolve_scope(&mut *self.pool.acquire().await?, &operation.access).await
    }
}

impl Run<ListTermbaseInfos> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ListTermbaseInfos) -> AppResult<Vec<TermbaseInfo>> {
        validate_query(&operation.query)?;

        let mut transaction = self.pool.begin().await?;

        let scope = read::resolve_scope(&mut transaction, &operation.access).await?;

        let comic_id = match scope {
            TermbaseScope::Global => None,
            TermbaseScope::Comic { comic_id } => Some(comic_id),
        };

        let query = key(&operation.query.text);

        let ids = sqlx::query_scalar!("SELECT id FROM termbase WHERE (scope = 'global' OR comic_id = ?) AND instr(name_key, ?) > 0 ORDER BY updated_at DESC, id LIMIT ? OFFSET ?",
            comic_id, query, operation.query.limit, operation.query.offset)
            .fetch_all(&mut *transaction).await?;

        let mut result = Vec::new();

        for id in ids {
            let base = read::base(&mut transaction, &id).await?;

            result.push(read::info(&mut transaction, base).await?);
        }

        transaction.rollback().await?;

        Ok(result)
    }
}

impl Run<ReadTermbase> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadTermbase) -> AppResult<TermbaseInfo> {
        let mut transaction = self.pool.begin().await?;

        let base = read::accessible_base(&mut transaction, &operation.access, &operation.id, false)
            .await?;

        let result = read::info(&mut transaction, base).await?;

        transaction.rollback().await?;

        Ok(result)
    }
}

impl Run<ListTermInfos> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ListTermInfos) -> AppResult<Vec<Term>> {
        validate_query(&operation.query)?;

        let mut transaction = self.pool.begin().await?;

        read::accessible_base(
            &mut transaction,
            &operation.access,
            &operation.termbase_id,
            false,
        )
        .await?;

        let query = key(&operation.query.text);

        let rows = sqlx::query_as!(read::TermRow, "SELECT id, termbase_id, source, targets, comment, created_at, updated_at FROM term WHERE termbase_id = ? AND instr(source_key, ?) > 0 ORDER BY updated_at DESC, id LIMIT ? OFFSET ?",
            operation.termbase_id, query, operation.query.limit, operation.query.offset)
            .fetch_all(&mut *transaction).await?;

        let result = rows.into_iter().map(Term::try_from).collect();

        transaction.rollback().await?;

        result
    }
}

#[cfg(test)]
mod validation_test;

#[cfg(test)]
mod import_test;

#[cfg(test)]
mod export_test;
