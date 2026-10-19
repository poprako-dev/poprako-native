use poprako_orchestra::Step;

use crate::data::chapter::ChapterMetadata;
use crate::data::comic::{ComicMetadata, NewComic};
use crate::data::editor::SavePageUnits;
use crate::data::import::{ImportMode, ImportParent};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::page::{matches_baseline, read_pages};
use crate::implementation::repository::read;
use crate::model::chapter::Chapter;
use crate::part::repository::comic::CreateComic;
use crate::part::repository::import::{ApplyChapterImport, CreateImportedChapter};
use crate::part::repository::operation::CreateChapter;
use crate::part::repository::operation::PersistPageUnits;
use crate::part::repository::page::AppendPages;
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

impl Step<CreateImportedChapter, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &CreateImportedChapter,
    ) -> AppResult<Chapter> {
        let input = &operation.input;

        validate_id(&input.chapter_id)?;

        let chapter_title = match input.metadata.subtitle.trim() {
            "" => "第1章".to_owned(),
            title => title.to_owned(),
        };

        let chapter = match &input.parent {
            ImportParent::New { comic_id } => {
                self.step(
                    context,
                    &CreateComic {
                        id: comic_id.clone(),
                        chapter_id: input.chapter_id.clone(),
                        input: NewComic {
                            comic: ComicMetadata {
                                title: input.metadata.title.clone(),
                                author: input.metadata.author.clone(),
                            },
                            chapter_title,
                        },
                    },
                )
                .await?
            }
            ImportParent::Existing { comic_id, baseline } => {
                self.step(
                    context,
                    &CreateChapter {
                        id: input.chapter_id.clone(),
                        comic_id: comic_id.clone(),
                        baseline: baseline.clone(),
                        metadata: ChapterMetadata {
                            title: chapter_title,
                        },
                    },
                )
                .await?
            }
        };

        self.step(
            context,
            &AppendPages {
                chapter_id: chapter.id.clone(),
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
                        chapter_id: chapter.id.clone(),
                        page_id: prepared.page.id.clone(),
                        expected_revision: 0,
                        units: prepared.units.clone(),
                    },
                },
            )
            .await?;

            // Imported initial content is revision zero until the entire creation commits.
            sqlx::query!(
                "UPDATE page SET unit_revision = 0 WHERE id = ? AND chapter_id = ?",
                prepared.page.id,
                chapter.id
            )
            .execute(&mut *context.0)
            .await?;
        }

        Ok(chapter)
    }
}

impl Step<ApplyChapterImport, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &ApplyChapterImport,
    ) -> AppResult<Vec<String>> {
        let input = &operation.input;

        let pages = read_pages(&mut context.0, &input.chapter_id).await?;

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
                            chapter_id: input.chapter_id.clone(),
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
