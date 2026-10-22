use poprako_orchestra::Step;
use uuid::Uuid;

use crate::complex::terminology::append::merge;
use crate::complex::terminology::key;
use crate::data::terminology::{TermMetadata, TermbaseAppendResult};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::terminology::read;
use crate::part::repository::terminology::{AppendTermbase, CreateTerm, UpdateTerm};
use crate::result::{AppError, AppResult};

impl Step<AppendTermbase, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &AppendTermbase,
    ) -> AppResult<TermbaseAppendResult> {
        read::accessible_base(
            &mut context.0,
            &operation.access,
            &operation.termbase_id,
            true,
        )
        .await?;

        let mut result = TermbaseAppendResult::default();

        for incoming in &operation.terms {
            let source_key = key(&incoming.source);

            let id = sqlx::query_scalar!(
                "SELECT id FROM term WHERE termbase_id = ? AND source_key = ?",
                operation.termbase_id,
                source_key
            )
            .fetch_optional(&mut *context.0)
            .await?;

            let Some(id) = id else {
                self.step(
                    context,
                    &CreateTerm {
                        access: operation.access.clone(),
                        termbase_id: operation.termbase_id.clone(),
                        id: Uuid::new_v4().to_string(),
                        metadata: incoming.clone(),
                    },
                )
                .await?;

                result.added += 1;

                continue;
            };

            let before = read::term(&mut context.0, &id).await?;

            let existing = TermMetadata {
                source: before.source.clone(),
                targets: before.targets.clone(),
                comment: before.comment.clone(),
            };

            let metadata = merge(existing.clone(), incoming.clone())?;

            if metadata == existing {
                result.unchanged += 1;

                continue;
            }

            self.step(
                context,
                &UpdateTerm {
                    access: operation.access.clone(),
                    baseline: before,
                    metadata,
                },
            )
            .await?;

            result.updated += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod recovery_test;
