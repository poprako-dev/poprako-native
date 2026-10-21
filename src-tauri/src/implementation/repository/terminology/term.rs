use poprako_orchestra::Step;

use crate::complex::terminology::{key, normalize_term};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::terminology::read;
use crate::model::terminology::Term;
use crate::part::repository::terminology::{CreateTerm, DeleteTerm, UpdateTerm};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

fn write_error(error: sqlx::Error) -> AppError {
    if error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
    {
        return AppError::DuplicateTerm;
    }

    error.into()
}

impl Step<CreateTerm, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &CreateTerm) -> AppResult<Term> {
        validate_id(&operation.id)?;

        let base = read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.termbase_id,
            true,
        )
        .await?;

        let metadata = normalize_term(operation.metadata.clone())?;

        let source_key = key(&metadata.source);

        let targets =
            serde_json::to_string(&metadata.targets).map_err(|_| AppError::InvalidInput)?;

        sqlx::query!("INSERT INTO term (id, termbase_id, source, source_key, targets, comment, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            operation.id, base.id, metadata.source, source_key, targets, metadata.comment, context.1, context.1)
            .execute(&mut *context.0).await.map_err(write_error)?;

        read::touch(&mut context.0, &base, context.1).await?;

        read::term(&mut context.0, &operation.id).await
    }
}

impl Step<UpdateTerm, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &UpdateTerm) -> AppResult<Term> {
        let base = read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.baseline.termbase_id,
            true,
        )
        .await?;

        let before = read::term(&mut context.0, &operation.baseline.id).await?;

        if before != operation.baseline {
            return Err(AppError::Conflict);
        }

        let metadata = normalize_term(operation.metadata.clone())?;

        if before.source == metadata.source
            && before.targets == metadata.targets
            && before.comment == metadata.comment
        {
            return Ok(before);
        }

        let source_key = key(&metadata.source);

        let targets =
            serde_json::to_string(&metadata.targets).map_err(|_| AppError::InvalidInput)?;

        sqlx::query!("UPDATE term SET source = ?, source_key = ?, targets = ?, comment = ?, updated_at = ? WHERE id = ?", metadata.source, source_key, targets, metadata.comment, context.1, before.id)
            .execute(&mut *context.0).await.map_err(write_error)?;

        read::touch(&mut context.0, &base, context.1).await?;

        read::term(&mut context.0, &before.id).await
    }
}

impl Step<DeleteTerm, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &DeleteTerm) -> AppResult<()> {
        let base = read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.baseline.termbase_id,
            true,
        )
        .await?;

        let before = read::term(&mut context.0, &operation.baseline.id).await?;

        if before != operation.baseline {
            return Err(AppError::Conflict);
        }

        sqlx::query!("DELETE FROM term WHERE id = ?", before.id)
            .execute(&mut *context.0)
            .await?;

        read::touch(&mut context.0, &base, context.1).await
    }
}
