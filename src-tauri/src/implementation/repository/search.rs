use std::collections::{BTreeMap, HashSet};

use poprako_orchestra::{Run, Step};

use crate::complex::text_transform::transform;
use crate::data::editor::{SavePageUnits, UnitDraft};
use crate::data::search::{ReplacementResult, SearchHit, TextStage};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::catalog::read_comic;
use crate::implementation::repository::read;
use crate::part::repository::operation::PersistPageUnits;
use crate::part::repository::search::{Replace, Search};
use crate::result::{AppError, AppResult};
use crate::value::validation::validate_id;

impl Run<Search> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &Search) -> AppResult<Vec<SearchHit>> {
        let input = &operation.input;

        validate_id(&input.comic_id)?;

        let query = input.query.trim();

        if query.is_empty() {
            return Err(AppError::InvalidInput);
        }

        let mut transaction = self.pool.begin().await?;

        read_comic(&mut transaction, &input.comic_id).await?;

        if query.contains('\0') {
            return Ok(Vec::new());
        }

        let proofreading = matches!(input.stage, TextStage::Proofreading);

        let rows = sqlx::query!(
            "SELECT u.id, u.page_id, u.position AS unit_index, p.position AS page_index, p.unit_revision,
            CASE WHEN ? THEN u.proofread_text ELSE u.translated_text END AS \"text!: String\"
            FROM unit u JOIN page p ON p.id = u.page_id
            WHERE p.comic_id = ? AND instr(CASE WHEN ? THEN u.proofread_text ELSE u.translated_text END, ?) > 0
            ORDER BY p.position, u.position LIMIT 101",
            proofreading, input.comic_id, proofreading, query
        ).fetch_all(&mut *transaction).await?;

        if rows.len() > 100 {
            return Err(AppError::InvalidInput);
        }

        transaction.commit().await?;

        rows.into_iter()
            .map(|row| {
                Ok(SearchHit {
                    unit_id: row.id,
                    page_id: row.page_id,
                    page_index: u32::try_from(row.page_index)
                        .map_err(|_| AppError::RecoveryRequired)?,
                    unit_index: u32::try_from(row.unit_index)
                        .map_err(|_| AppError::RecoveryRequired)?,
                    unit_revision: row.unit_revision,
                    text: row.text,
                })
            })
            .collect()
    }
}

impl Step<Replace, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &Replace,
    ) -> AppResult<ReplacementResult> {
        let input = &operation.input;

        validate_id(&input.comic_id)?;

        if input.units.is_empty() || input.units.len() > 100 {
            return Err(AppError::InvalidInput);
        }

        let mut seen = HashSet::new();

        let mut grouped = BTreeMap::new();

        for replacement in &input.units {
            validate_id(&replacement.hit.unit_id)?;

            validate_id(&replacement.hit.page_id)?;

            if !seen.insert(&replacement.hit.unit_id) {
                return Err(AppError::InvalidInput);
            }

            grouped
                .entry(&replacement.hit.page_id)
                .or_insert_with(Vec::new)
                .push(replacement);
        }

        let mut result = ReplacementResult {
            changed_unit_count: 0,
            changed_page_ids: Vec::new(),
        };

        for (page_id, replacements) in grouped {
            let editor = read::editor(&mut context.0, &input.comic_id, page_id).await?;

            let mut drafts: Vec<_> = editor.units.iter().map(UnitDraft::from).collect();

            let mut changed = false;

            for replacement in replacements {
                if replacement.hit.unit_revision != editor.page.unit_revision {
                    return Err(AppError::Conflict);
                }

                let draft = drafts
                    .iter_mut()
                    .find(|draft| draft.id == replacement.hit.unit_id)
                    .ok_or(AppError::Conflict)?;

                let text = match input.stage {
                    TextStage::Translation => &mut draft.translated_text,
                    TextStage::Proofreading => &mut draft.proofread_text,
                };

                if *text != replacement.hit.text {
                    return Err(AppError::Conflict);
                }

                let transformed = transform(text, &replacement.rules)?;

                if transformed != *text {
                    *text = transformed;

                    changed = true;

                    result.changed_unit_count += 1;
                }
            }

            if !changed {
                continue;
            }

            self.step(
                context,
                &PersistPageUnits {
                    snapshot: SavePageUnits {
                        comic_id: input.comic_id.clone(),
                        page_id: page_id.clone(),
                        expected_revision: editor.page.unit_revision,
                        units: drafts,
                    },
                },
            )
            .await?;

            result.changed_page_ids.push(page_id.clone());
        }

        Ok(result)
    }
}
