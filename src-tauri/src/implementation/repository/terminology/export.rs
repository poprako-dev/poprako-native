use poprako_orchestra::Run;

use crate::data::terminology::portable::{PortableTerm, PortableTermbase};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::terminology::read;
use crate::model::terminology::Term;
use crate::part::repository::terminology::ReadTermbaseExport;
use crate::result::{AppError, AppResult};

fn optional_text(value: String) -> Option<String> {
    match value.is_empty() {
        true => None,
        false => Some(value),
    }
}

impl Run<ReadTermbaseExport> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadTermbaseExport) -> AppResult<PortableTermbase> {
        let mut transaction = self.pool.begin().await?;

        let base = read::accessible_base(&mut transaction, &operation.access, &operation.id, false)
            .await?;

        let rows = sqlx::query_as!(read::TermRow, "SELECT id, termbase_id, source, targets, comment, created_at, updated_at FROM term WHERE termbase_id = ? ORDER BY source COLLATE BINARY, id", operation.id)
            .fetch_all(&mut *transaction).await?;

        let mut terms = Vec::with_capacity(rows.len());

        for row in rows {
            let term = Term::try_from(row)?;

            terms.push(PortableTerm {
                source: term.source,
                targets: term.targets,
                comment: optional_text(term.comment),
            });
        }

        transaction.rollback().await?;

        Ok(PortableTermbase {
            name: base.name,
            description: optional_text(base.description),
            terms,
        })
    }
}
