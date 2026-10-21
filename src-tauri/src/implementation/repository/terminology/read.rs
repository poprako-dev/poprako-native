use sqlx::SqliteConnection;

use crate::data::terminology::{TermbaseInfo, TerminologyContext};
use crate::model::terminology::{Term, Termbase, TermbaseScope};
use crate::result::{AppError, AppResult};
use crate::value::validation::{MAX_SAFE_INTEGER, validate_id};

pub struct BaseRow {
    pub id: String,
    pub scope: String,
    pub comic_id: Option<String>,
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<BaseRow> for Termbase {
    type Error = AppError;

    fn try_from(row: BaseRow) -> AppResult<Self> {
        let scope = match (row.scope.as_str(), row.comic_id) {
            ("global", None) => TermbaseScope::Global,
            ("comic", Some(comic_id)) => TermbaseScope::Comic { comic_id },
            _ => return Err(AppError::RecoveryRequired),
        };

        if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&row.created_at)
            || !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&row.updated_at)
        {
            return Err(AppError::RecoveryRequired);
        }

        Ok(Self {
            id: row.id,
            scope,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub struct TermRow {
    pub id: String,
    pub termbase_id: String,
    pub source: String,
    pub targets: String,
    pub comment: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<TermRow> for Term {
    type Error = AppError;

    fn try_from(row: TermRow) -> AppResult<Self> {
        let targets: Vec<String> =
            serde_json::from_str(&row.targets).map_err(|_| AppError::RecoveryRequired)?;

        let metadata =
            crate::complex::terminology::normalize_term(crate::data::terminology::TermMetadata {
                source: row.source.clone(),
                targets: targets.clone(),
                comment: row.comment.clone(),
            })
            .map_err(|_| AppError::RecoveryRequired)?;

        if metadata.source != row.source
            || metadata.targets != targets
            || metadata.comment != row.comment
            || !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&row.created_at)
            || !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&row.updated_at)
        {
            return Err(AppError::RecoveryRequired);
        }

        Ok(Self {
            id: row.id,
            termbase_id: row.termbase_id,
            source: row.source,
            targets,
            comment: row.comment,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// # Errors
/// Resolves current chapter ownership instead of trusting a caller-supplied comic.
pub async fn resolve_scope(
    connection: &mut SqliteConnection,
    access: &TerminologyContext,
) -> AppResult<TermbaseScope> {
    match access {
        TerminologyContext::Global => Ok(TermbaseScope::Global),
        TerminologyContext::Chapter { chapter_id } => {
            validate_id(chapter_id)?;

            let comic_id =
                sqlx::query_scalar!("SELECT comic_id FROM chapter WHERE id = ?", chapter_id)
                    .fetch_one(connection)
                    .await?;

            Ok(TermbaseScope::Comic { comic_id })
        }
    }
}

/// # Errors
/// Reads a library without changing timestamps.
pub async fn base(connection: &mut SqliteConnection, id: &str) -> AppResult<Termbase> {
    validate_id(id)?;

    sqlx::query_as!(BaseRow, "SELECT id, scope, comic_id, name, description, created_at, updated_at FROM termbase WHERE id = ?", id)
        .fetch_one(connection).await?.try_into()
}

/// # Errors
/// Enforces read visibility and restricts chapter-context writes to the current comic.
pub async fn accessible_base(
    connection: &mut SqliteConnection,
    access: &TerminologyContext,
    id: &str,
    writing: bool,
) -> AppResult<Termbase> {
    let expected = resolve_scope(connection, access).await?;

    let value = base(connection, id).await?;

    if value.scope != expected && (writing || value.scope != TermbaseScope::Global) {
        return Err(AppError::NotFound);
    }

    Ok(value)
}

/// # Errors
/// Reads complete ordered translations; malformed stored content is not repaired silently.
pub async fn term(connection: &mut SqliteConnection, id: &str) -> AppResult<Term> {
    validate_id(id)?;

    sqlx::query_as!(TermRow, "SELECT id, termbase_id, source, targets, comment, created_at, updated_at FROM term WHERE id = ?", id)
        .fetch_one(connection).await?.try_into()
}

/// # Errors
/// Returns a derived count in the same read snapshot as the library.
pub async fn info(
    connection: &mut SqliteConnection,
    termbase: Termbase,
) -> AppResult<TermbaseInfo> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM term WHERE termbase_id = ?",
        termbase.id
    )
    .fetch_one(connection)
    .await?;

    Ok(TermbaseInfo {
        termbase,
        term_count: u32::try_from(count).map_err(|_| AppError::RecoveryRequired)?,
    })
}

/// # Errors
/// Changes library and comic time only; chapter content and editor versions remain independent.
pub async fn touch(
    connection: &mut SqliteConnection,
    termbase: &Termbase,
    now: i64,
) -> AppResult<()> {
    sqlx::query!(
        "UPDATE termbase SET updated_at = ? WHERE id = ?",
        now,
        termbase.id
    )
    .execute(&mut *connection)
    .await?;

    if let TermbaseScope::Comic { comic_id } = &termbase.scope {
        crate::implementation::repository::library::touch_comic(connection, comic_id, now).await?;
    }

    Ok(())
}
