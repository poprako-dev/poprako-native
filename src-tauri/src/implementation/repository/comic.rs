use poprako_orchestra::Step;

use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::model::comic::Comic;
use crate::part::repository::operation::CreateComic;
use crate::result::{AppError, AppResult};

impl Step<CreateComic, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &CreateComic) -> AppResult<Comic> {
        crate::value::validation::validate_id(&operation.id)?;

        let title = operation.metadata.title.trim().to_owned();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let comic = Comic {
            id: operation.id.clone(),
            title,
            subtitle: operation.metadata.subtitle.trim().to_owned(),
            author: operation.metadata.author.trim().to_owned(),
            created_at: context.1,
            updated_at: 0,
        };

        let comic = Comic {
            updated_at: comic.created_at,
            ..comic
        };

        sqlx::query!(
            "INSERT INTO comic (id, title, subtitle, author, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            comic.id, comic.title, comic.subtitle, comic.author, comic.created_at, comic.updated_at
        ).execute(&mut *context.0).await?;

        Ok(comic)
    }
}
