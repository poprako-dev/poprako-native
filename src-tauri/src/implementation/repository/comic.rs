use poprako_orchestra::Step;

use crate::data::chapter::ChapterMetadata;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::library::read_comic;
use crate::model::chapter::Chapter;
use crate::model::comic::Comic;
use crate::part::repository::comic::{CreateComic, DeleteComic, UpdateComicMetadata};
use crate::part::repository::operation::CreateChapter;
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

impl Step<CreateComic, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &CreateComic,
    ) -> AppResult<Chapter> {
        validate_id(&operation.id)?;

        let title = operation.input.comic.title.trim();

        let author = operation.input.comic.author.trim();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let now = context.1;

        sqlx::query!(
            "INSERT INTO comic (id, title, author, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            operation.id,
            title,
            author,
            now,
            now
        )
        .execute(&mut *context.0)
        .await?;

        self.step(
            context,
            &CreateChapter {
                id: operation.chapter_id.clone(),
                comic_id: operation.id.clone(),
                baseline: Vec::new(),
                metadata: ChapterMetadata {
                    title: operation.input.chapter_title.clone(),
                },
            },
        )
        .await
    }
}

impl Step<UpdateComicMetadata, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &UpdateComicMetadata,
    ) -> AppResult<Comic> {
        let before = read_comic(&mut context.0, &operation.comic_id).await?;

        let title = operation.metadata.title.trim();

        let author = operation.metadata.author.trim();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        if before.title == title && before.author == author {
            return Ok(before);
        }

        let now = context.1;

        sqlx::query!(
            "UPDATE comic SET title = ?, author = ?, updated_at = ? WHERE id = ?",
            title,
            author,
            now,
            operation.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read_comic(&mut context.0, &operation.comic_id).await
    }
}

impl Step<DeleteComic, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(&self, context: &mut SqliteContext, operation: &DeleteComic) -> AppResult<()> {
        read_comic(&mut context.0, &operation.comic_id).await?;

        sqlx::query!("DELETE FROM comic WHERE id = ?", operation.comic_id)
            .execute(&mut *context.0)
            .await?;

        Ok(())
    }
}
