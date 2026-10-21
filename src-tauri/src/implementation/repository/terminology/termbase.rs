use poprako_orchestra::Step;

use crate::complex::terminology::{key, normalize_base};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::terminology::read;
use crate::model::terminology::{Termbase, TermbaseScope};
use crate::part::repository::terminology::{CreateTermbase, DeleteTermbase, UpdateTermbase};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

fn write_error(error: sqlx::Error) -> AppError {
    if error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
    {
        return AppError::DuplicateTermbase;
    }

    error.into()
}

impl Step<CreateTermbase, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &CreateTermbase,
    ) -> AppResult<Termbase> {
        validate_id(&operation.id)?;

        let scope = read::resolve_scope(&mut context.0, &operation.access).await?;

        if scope != operation.scope {
            return Err(AppError::Conflict);
        }

        let (kind, comic_id) = match &scope {
            TermbaseScope::Global => ("global", None),
            TermbaseScope::Comic { comic_id } => ("comic", Some(comic_id)),
        };

        let metadata = normalize_base(&operation.metadata)?;

        let name_key = key(&metadata.name);

        sqlx::query!("INSERT INTO termbase (id, scope, comic_id, name, name_key, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            operation.id, kind, comic_id, metadata.name, name_key, metadata.description, context.1, context.1)
            .execute(&mut *context.0).await.map_err(write_error)?;

        let base = read::base(&mut context.0, &operation.id).await?;

        read::touch(&mut context.0, &base, context.1).await?;

        Ok(base)
    }
}

impl Step<UpdateTermbase, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &UpdateTermbase,
    ) -> AppResult<Termbase> {
        let base = read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.baseline.id,
            true,
        )
        .await?;

        if base != operation.baseline {
            return Err(AppError::Conflict);
        }

        let metadata = normalize_base(&operation.metadata)?;

        if base.name == metadata.name && base.description == metadata.description {
            return Ok(base);
        }

        let name_key = key(&metadata.name);

        sqlx::query!(
            "UPDATE termbase SET name = ?, name_key = ?, description = ? WHERE id = ?",
            metadata.name,
            name_key,
            metadata.description,
            base.id
        )
        .execute(&mut *context.0)
        .await
        .map_err(write_error)?;

        read::touch(&mut context.0, &base, context.1).await?;

        read::base(&mut context.0, &base.id).await
    }
}

impl Step<DeleteTermbase, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &DeleteTermbase) -> AppResult<()> {
        let base = read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.baseline.id,
            true,
        )
        .await?;

        if base != operation.baseline {
            return Err(AppError::Conflict);
        }

        read::touch(&mut context.0, &base, context.1).await?;

        sqlx::query!("DELETE FROM termbase WHERE id = ?", base.id)
            .execute(&mut *context.0)
            .await?;

        Ok(())
    }
}
