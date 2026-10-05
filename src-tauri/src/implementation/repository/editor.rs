use std::collections::{HashMap, HashSet};

use poprako_orchestra::{Run, Step};

use crate::data::editor::{PageEditor, UnitDraft};
use crate::implementation::coordinator::{Immediate, SqliteContext};
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::read;
use crate::model::unit::Unit;
use crate::part::repository::operation::{PersistPageUnits, ReadPageEditor};
use crate::result::{AppError, AppResult};
use crate::value::validation::{
    MAX_SAFE_INTEGER, normalize_body, position, validate_coordinate, validate_id,
};

fn normalize(units: &[UnitDraft]) -> AppResult<Vec<UnitDraft>> {
    position(units.len())?;

    let mut seen = HashSet::new();

    units
        .iter()
        .map(|unit| {
            validate_id(&unit.id)?;

            validate_coordinate(unit.x_coord)?;

            validate_coordinate(unit.y_coord)?;

            if !seen.insert(&unit.id) {
                return Err(AppError::InvalidInput);
            }

            Ok(UnitDraft {
                translated_text: normalize_body(&unit.translated_text),
                proofread_text: normalize_body(&unit.proofread_text),
                ..unit.clone()
            })
        })
        .collect()
}

fn persisted(draft: &UnitDraft, index: u32, page_id: &str, old: Option<&Unit>, now: i64) -> Unit {
    let changed = old.is_none_or(|unit| unit.index != index || UnitDraft::from(unit) != *draft);

    let updated_at = match (old, changed) {
        (Some(unit), false) => unit.updated_at,
        _ => now,
    };

    Unit {
        id: draft.id.clone(),
        page_id: page_id.to_owned(),
        index,
        x_coord: draft.x_coord,
        y_coord: draft.y_coord,
        is_bubble: draft.is_bubble,
        is_flagged: draft.is_flagged,
        translated_text: draft.translated_text.clone(),
        proofread_text: draft.proofread_text.clone(),
        is_proofread: draft.is_proofread,
        created_at: old.map_or(now, |unit| unit.created_at),
        updated_at,
    }
}

async fn write_difference(
    context: &mut SqliteContext,
    page_id: &str,
    before: &PageEditor,
    target: &[UnitDraft],
    final_units: &[Unit],
    old: &HashMap<&str, &Unit>,
) -> AppResult<()> {
    let ids: HashSet<_> = target.iter().map(|unit| unit.id.as_str()).collect();

    for unit in &before.units {
        if !ids.contains(unit.id.as_str()) {
            sqlx::query!(
                "UPDATE work_position SET unit_id = NULL WHERE unit_id = ?",
                unit.id
            )
            .execute(&mut *context.0)
            .await?;

            sqlx::query!(
                "DELETE FROM unit WHERE id = ? AND page_id = ?",
                unit.id,
                page_id
            )
            .execute(&mut *context.0)
            .await?;
        }
    }

    let offset = i64::try_from(before.units.len().max(target.len()))
        .map_err(|_| AppError::InvalidInput)?
        .checked_add(1)
        .ok_or(AppError::InvalidInput)?;

    for unit in final_units {
        if let Some(previous) = old
            .get(unit.id.as_str())
            .filter(|previous| previous.index != unit.index)
        {
            let temporary = offset
                .checked_add(i64::from(previous.index))
                .ok_or(AppError::InvalidInput)?;

            sqlx::query!(
                "UPDATE unit SET position = ? WHERE id = ?",
                temporary,
                unit.id
            )
            .execute(&mut *context.0)
            .await?;
        }
    }

    for unit in final_units {
        if old
            .get(unit.id.as_str())
            .is_some_and(|previous| **previous == *unit)
        {
            continue;
        }

        let position = i64::from(unit.index);

        match old.contains_key(unit.id.as_str()) {
            true => {
                sqlx::query!("UPDATE unit SET position = ?, x_coord = ?, y_coord = ?, is_bubble = ?, is_flagged = ?, translated_text = ?, proofread_text = ?, is_proofread = ?, updated_at = ? WHERE id = ? AND page_id = ?",
                        position, unit.x_coord, unit.y_coord, unit.is_bubble, unit.is_flagged, unit.translated_text, unit.proofread_text, unit.is_proofread, unit.updated_at, unit.id, page_id)
                        .execute(&mut *context.0).await?;
            }
            false => {
                sqlx::query!("INSERT INTO unit (id, page_id, position, x_coord, y_coord, is_bubble, is_flagged, translated_text, proofread_text, is_proofread, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        unit.id, page_id, position, unit.x_coord, unit.y_coord, unit.is_bubble, unit.is_flagged, unit.translated_text, unit.proofread_text, unit.is_proofread, unit.created_at, unit.updated_at)
                        .execute(&mut *context.0).await?;
            }
        }
    }

    Ok(())
}

impl Run<ReadPageEditor> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadPageEditor) -> AppResult<PageEditor> {
        validate_id(&operation.comic_id)?;

        validate_id(&operation.page_id)?;

        let mut transaction = self.pool.begin().await?;

        let result =
            read::editor(&mut transaction, &operation.comic_id, &operation.page_id).await?;

        transaction.commit().await?;

        Ok(result)
    }
}

impl Step<PersistPageUnits, SqliteContext> for SqliteRepository {
    type Level = Immediate;
    type Error = AppError;

    async fn step(
        &self,
        context: &mut SqliteContext,
        operation: &PersistPageUnits,
    ) -> AppResult<PageEditor> {
        let input = &operation.snapshot;

        validate_id(&input.comic_id)?;

        validate_id(&input.page_id)?;

        let target = normalize(&input.units)?;

        let before = read::editor(&mut context.0, &input.comic_id, &input.page_id).await?;

        if before.page.unit_revision != input.expected_revision {
            return Err(AppError::Conflict);
        }

        if before
            .units
            .iter()
            .map(UnitDraft::from)
            .eq(target.iter().cloned())
        {
            return Ok(before);
        }

        let revision = before
            .page
            .unit_revision
            .checked_add(1)
            .filter(|value| *value <= MAX_SAFE_INTEGER)
            .ok_or(AppError::RecoveryRequired)?;

        let now = context.1;

        let old: HashMap<_, _> = before
            .units
            .iter()
            .map(|unit| (unit.id.as_str(), unit))
            .collect();

        let final_units: Vec<_> = target
            .iter()
            .enumerate()
            .map(|(index, draft)| {
                Ok(persisted(
                    draft,
                    position(index)?,
                    &input.page_id,
                    old.get(draft.id.as_str()).copied(),
                    now,
                ))
            })
            .collect::<AppResult<_>>()?;

        write_difference(
            context,
            &input.page_id,
            &before,
            &target,
            &final_units,
            &old,
        )
        .await?;

        sqlx::query!(
            "UPDATE page SET unit_revision = ?, updated_at = ? WHERE id = ?",
            revision,
            now,
            input.page_id
        )
        .execute(&mut *context.0)
        .await?;

        sqlx::query!(
            "UPDATE comic SET updated_at = ? WHERE id = ?",
            now,
            input.comic_id
        )
        .execute(&mut *context.0)
        .await?;

        read::editor(&mut context.0, &input.comic_id, &input.page_id).await
    }
}
