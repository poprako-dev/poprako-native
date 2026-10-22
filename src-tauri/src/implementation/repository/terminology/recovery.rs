use sqlx::SqliteConnection;

use crate::data::recovery::RecoveryScope;
use crate::implementation::repository::terminology::read;
use crate::model::terminology::{Term, Termbase};
use crate::result::{AppError, AppResult};

/// # Errors
/// Captures exactly the library aggregates or terms participating in the transaction.
pub async fn capture(
    connection: &mut SqliteConnection,
    scope: &RecoveryScope,
) -> AppResult<(Vec<Termbase>, Vec<Term>)> {
    let base_ids =
        serde_json::to_string(&scope.termbase_ids).map_err(|_| AppError::RecoveryRequired)?;

    let comic_ids =
        serde_json::to_string(&scope.comic_ids).map_err(|_| AppError::RecoveryRequired)?;

    let term_ids =
        serde_json::to_string(&scope.term_ids).map_err(|_| AppError::RecoveryRequired)?;

    let base_rows = sqlx::query_as!(read::BaseRow, "SELECT id, scope, comic_id, name, description, created_at, updated_at FROM termbase WHERE id IN (SELECT value FROM json_each(?)) OR (? AND comic_id IN (SELECT value FROM json_each(?))) ORDER BY id", base_ids, scope.comic_terminology, comic_ids)
        .fetch_all(&mut *connection).await?;

    let termbases = base_rows
        .into_iter()
        .map(Termbase::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    let rows = sqlx::query_as!(read::TermRow, "SELECT id, termbase_id, source, targets, comment, created_at, updated_at FROM term WHERE id IN (SELECT value FROM json_each(?)) OR (? AND termbase_id IN (SELECT value FROM json_each(?))) OR (? AND termbase_id IN (SELECT id FROM termbase WHERE comic_id IN (SELECT value FROM json_each(?)))) ORDER BY id", term_ids, scope.termbase_terms, base_ids, scope.comic_terminology, comic_ids)
        .fetch_all(connection).await?;

    let terms = rows
        .into_iter()
        .map(Term::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok((termbases, terms))
}
