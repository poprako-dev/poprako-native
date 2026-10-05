use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use uuid::Uuid;

use crate::data::archive_task::{ArchivePagePreview, ArchivePreview, ArchiveTarget, ArchiveTask};
use crate::data::comic::ComicMetadata;
use crate::data::page::PageBaseline;
use crate::data::search::TextStage;
use crate::harness::Harness;
use crate::implementation::archive::source;
use crate::implementation::archive_task::picker::ImportSelection;
use crate::implementation::archive_task::{ImportArtifact, check_cancel};
use crate::result::{AppError, AppResult};
use crate::value::validation::position;

fn preview(artifact: &ImportArtifact) -> AppResult<ArchivePreview> {
    let (document, targets) = match artifact {
        ImportArtifact::New(prepared) => (
            &prepared.document,
            prepared
                .image_names
                .iter()
                .map(|name| (name.clone(), 0))
                .collect::<Vec<_>>(),
        ),
        ImportArtifact::Existing {
            document,
            target_pages,
            ..
        } => (document, target_pages.clone()),
    };

    let count = document.pages.iter().try_fold(0_usize, |total, page| {
        total
            .checked_add(page.units.len())
            .ok_or(AppError::InvalidInput)
    })?;

    Ok(ArchivePreview {
        pages: document
            .pages
            .iter()
            .zip(targets)
            .map(|(page, (target_image_name, target_unit_count))| {
                let source_image_name = page
                    .image
                    .as_ref()
                    .map(|image| image.original_name.as_str())
                    .or(page.source_image_path.as_deref())
                    .and_then(|name| name.rsplit('/').next())
                    .unwrap_or_default()
                    .to_owned();

                Ok(ArchivePagePreview {
                    index: page.index,
                    source_image_name,
                    target_image_name,
                    source_unit_count: position(page.units.len())?,
                    target_unit_count,
                })
            })
            .collect::<AppResult<Vec<_>>>()?,
        metadata: ComicMetadata {
            title: document.comic.title.clone(),
            subtitle: document.comic.subtitle.clone(),
            author: document.comic.author.clone(),
        },
        page_count: position(document.pages.len())?,
        unit_count: position(count)?,
        warnings: document.warnings.clone(),
    })
}

/// # Errors
/// Discards only unpublished prepared files, retaining user inputs untouched.
pub fn discard(state: &Harness, artifact: &ImportArtifact) -> AppResult<()> {
    if let ImportArtifact::New(prepared) = artifact {
        for image in &prepared.images {
            state
                .resource
                .discard_prepared(image)
                .map_err(|_| AppError::Storage)?;
        }
    }

    Ok(())
}

/// # Errors
/// Cancels preparation or disposes an uncommitted preview without touching originals.
pub async fn cancel(state: Arc<Harness>, id: String) -> AppResult<()> {
    state.archive_tasks.cancel(&id)?;

    let artifact = state.archive_tasks.update(&id, |record| {
        if matches!(record.status, ArchiveTask::AwaitingConfirmation { .. }) {
            record.status = ArchiveTask::Cleaning;

            return Ok(record.artifact.take());
        }

        Ok(None)
    })?;

    if let Some(artifact) = artifact {
        let worker = Arc::clone(&state);

        let result = tokio::task::spawn_blocking(move || discard(&worker, &artifact))
            .await
            .map_err(|_| AppError::Storage)?;

        state.archive_tasks.update(&id, |record| {
            record.status = match &result {
                Ok(()) => ArchiveTask::Cancelled,
                Err(error) => ArchiveTask::Failed {
                    message: error.to_string(),
                    commit_uncertain: false,
                },
            };

            record.permit = None;

            Ok(())
        })?;

        result?;
    }

    Ok(())
}

async fn build(
    state: Arc<Harness>,
    id: String,
    selection: ImportSelection,
    target: ArchiveTarget,
    text_stage: TextStage,
    cancel: Arc<AtomicBool>,
) -> AppResult<ImportArtifact> {
    match target {
        ArchiveTarget::New => tokio::task::spawn_blocking(move || {
            let mut progress = |completed, total| {
                check_cancel(&cancel)?;

                state.archive_tasks.update(&id, |record| {
                    record.status = ArchiveTask::Preparing {
                        completed: position(completed)?,
                        total: position(total)?,
                    };

                    Ok(())
                })
            };

            let prepared = source::prepare_controlled(
                &selection.source,
                &selection.images,
                &state.directory().join("staging"),
                &state.resource,
                &state.image,
                &text_stage,
                &mut progress,
            )?;

            Ok(ImportArtifact::New(prepared))
        })
        .await
        .map_err(|_| AppError::Storage)?,
        ArchiveTarget::Existing { comic_id } => {
            let snapshot =
                crate::usecase::export::snapshot(state.database().await?, &comic_id).await?;

            tokio::task::spawn_blocking(move || {
                let document = source::read_translation(&selection.source, &text_stage)?;

                if document.pages.len() != snapshot.pages.len() {
                    return Err(AppError::InvalidInput);
                }

                let mut baseline = Vec::new();

                let mut target_pages = Vec::new();

                for editor in &snapshot.pages {
                    check_cancel(&cancel)?;

                    let page = &editor.page;

                    let lease = state
                        .resource
                        .open_original(
                            &page.image.reference,
                            Uuid::parse_str(&comic_id).map_err(|_| AppError::InvalidInput)?,
                            Uuid::parse_str(&page.id).map_err(|_| AppError::InvalidInput)?,
                        )
                        .map_err(|_| AppError::Storage)?;

                    lease.file.metadata().map_err(|_| AppError::Storage)?;

                    let path = state
                        .resource
                        .resolve(
                            &page.image.reference,
                            Uuid::parse_str(&comic_id).map_err(|_| AppError::InvalidInput)?,
                            Uuid::parse_str(&page.id).map_err(|_| AppError::InvalidInput)?,
                        )
                        .map_err(|_| AppError::Storage)?;

                    state.image.release().map_err(|_| AppError::Storage)?;

                    state
                        .image
                        .with_image(&path, |_| Ok(()))
                        .map_err(|_| AppError::Storage)?;

                    baseline.push(PageBaseline {
                        id: page.id.clone(),
                        unit_revision: page.unit_revision,
                        image_reference: page.image.reference.clone(),
                    });

                    target_pages.push((
                        page.image.original_name.clone(),
                        position(editor.units.len())?,
                    ));
                }

                Ok(ImportArtifact::Existing {
                    document,
                    comic_id,
                    baseline,
                    target_pages,
                })
            })
            .await
            .map_err(|_| AppError::Storage)?
        }
    }
}

/// Runs independently of the command response and leaves a queryable terminal outcome.
pub async fn run(
    state: Arc<Harness>,
    id: String,
    selection: ImportSelection,
    target: ArchiveTarget,
    text_stage: TextStage,
    cancel: Arc<AtomicBool>,
) {
    let result = build(
        Arc::clone(&state),
        id.clone(),
        selection,
        target,
        text_stage,
        Arc::clone(&cancel),
    )
    .await;

    let outcome = match result {
        Ok(artifact) => match cancel.load(Ordering::Acquire) {
            true => {
                let discarded = discard(&state, &artifact);

                match discarded {
                    Ok(()) => state.archive_tasks.update(&id, |record| {
                        record.status = ArchiveTask::Cancelled;

                        record.permit = None;

                        Ok(())
                    }),
                    Err(error) => Err(error),
                }
            }
            false => match preview(&artifact) {
                Ok(preview) => state.archive_tasks.update(&id, |record| {
                    record.status = ArchiveTask::AwaitingConfirmation { preview };

                    record.artifact = Some(artifact);

                    Ok(())
                }),
                Err(error) => Err(error),
            },
        },
        Err(error) => state.archive_tasks.update(&id, |record| {
            record.status = match cancel.load(Ordering::Acquire) {
                true => ArchiveTask::Cancelled,
                false => ArchiveTask::Failed {
                    message: error.to_string(),
                    commit_uncertain: false,
                },
            };

            record.permit = None;

            Ok(())
        }),
    };

    if let Err(error) = outcome {
        let recorded = state.archive_tasks.update(&id, |record| {
            record.status = ArchiveTask::Failed {
                message: error.to_string(),
                commit_uncertain: false,
            };

            record.permit = None;

            Ok(())
        });

        if recorded.is_err() {
            eprintln!("Archive preparation state could not be recorded");
        }
    }

    if cancel.load(Ordering::Acquire)
        && matches!(
            state.archive_tasks.get(&id),
            Ok(ArchiveTask::AwaitingConfirmation { .. })
        )
        && self::cancel(state, id).await.is_err()
    {
        eprintln!("Cancelled archive preview cleanup failed");
    }
}
