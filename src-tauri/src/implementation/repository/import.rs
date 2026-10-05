use poprako_orchestra::Step;

use crate::data::editor::SavePageUnits;
use crate::data::import::ImportMode;
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::page::{matches_baseline, read_pages};
use crate::implementation::repository::read;
use crate::model::comic::Comic;
use crate::part::repository::import::{ApplyComicImport, CreateImportedComic};
use crate::part::repository::operation::PersistPageUnits;
use crate::part::repository::page::AppendPages;
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

impl Step<CreateImportedComic, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &CreateImportedComic,
    ) -> AppResult<Comic> {
        let input = &operation.input;

        validate_id(&input.comic_id)?;

        let title = input.metadata.title.trim().to_owned();

        if title.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let comic = Comic {
            id: input.comic_id.clone(),
            title,
            subtitle: input.metadata.subtitle.trim().to_owned(),
            author: input.metadata.author.trim().to_owned(),
            created_at: context.1,
            updated_at: context.1,
        };

        sqlx::query!("INSERT INTO comic (id, title, subtitle, author, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)", comic.id, comic.title, comic.subtitle, comic.author, comic.created_at, comic.updated_at)
            .execute(&mut *context.0).await?;

        self.step(
            context,
            &AppendPages {
                comic_id: comic.id.clone(),
                baseline: Vec::new(),
                pages: input.pages.iter().map(|page| page.page.clone()).collect(),
            },
        )
        .await?;

        for prepared in &input.pages {
            self.step(
                context,
                &PersistPageUnits {
                    snapshot: SavePageUnits {
                        comic_id: comic.id.clone(),
                        page_id: prepared.page.id.clone(),
                        expected_revision: 0,
                        units: prepared.units.clone(),
                    },
                },
            )
            .await?;

            // Imported initial content is revision zero until the entire creation commits.
            sqlx::query!(
                "UPDATE page SET unit_revision = 0 WHERE id = ? AND comic_id = ?",
                prepared.page.id,
                comic.id
            )
            .execute(&mut *context.0)
            .await?;
        }

        Ok(comic)
    }
}

impl Step<ApplyComicImport, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ApplyComicImport,
    ) -> AppResult<Vec<String>> {
        let input = &operation.input;

        let pages = read_pages(&mut context.0, &input.comic_id).await?;

        if pages.len() != input.baseline.len()
            || !pages
                .iter()
                .zip(&input.baseline)
                .all(|(page, baseline)| matches_baseline(page, baseline))
        {
            return Err(AppError::Conflict);
        }

        if pages.len() != input.pages.len() {
            return Err(AppError::InvalidInput);
        }

        let mut affected = Vec::new();

        for (page, units) in pages.iter().zip(&input.pages) {
            if input.mode == ImportMode::FillEmpty
                && !read::units(&mut context.0, &page.id).await?.is_empty()
            {
                continue;
            }

            for unit in units {
                let existing = sqlx::query!("SELECT id FROM unit WHERE id = ?", unit.id)
                    .fetch_optional(&mut *context.0)
                    .await?;

                if existing.is_some() {
                    return Err(AppError::Conflict);
                }
            }

            let result = self
                .step(
                    context,
                    &PersistPageUnits {
                        snapshot: SavePageUnits {
                            comic_id: input.comic_id.clone(),
                            page_id: page.id.clone(),
                            expected_revision: page.unit_revision,
                            units: units.clone(),
                        },
                    },
                )
                .await?;

            if result.page.unit_revision != page.unit_revision {
                affected.push(page.id.clone());
            }
        }

        Ok(affected)
    }
}
