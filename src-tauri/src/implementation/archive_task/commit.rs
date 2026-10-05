use std::sync::Arc;

use crate::data::archive_task::{ArchiveCompletion, ArchiveImportMode, ArchiveTask};
use crate::data::comic::ComicMetadata;
use crate::data::import::ImportMode;
use crate::harness::Harness;
use crate::implementation::archive::import;
use crate::implementation::archive_task::{ImportArtifact, UncertainArtifact};
use crate::result::{AppError, AppResult};

async fn apply(
    state: Arc<Harness>,
    id: &str,
    artifact: ImportArtifact,
    metadata: ComicMetadata,
    mode: ArchiveImportMode,
) -> AppResult<ArchiveCompletion> {
    match artifact {
        ImportArtifact::New(prepared) => {
            let worker = Arc::clone(&state);

            let published = tokio::task::spawn_blocking(move || {
                import::publish(prepared, metadata, &worker.resource)
            })
            .await
            .map_err(|_| AppError::Storage)??;

            let comic_id = published.input.comic_id.clone();

            let result = crate::usecase::import::create_comic_from_import(
                state.database().await?,
                published.input.clone(),
            )
            .await;

            if matches!(
                result,
                Err(AppError::CommitUncertain | AppError::RecoveryRequired)
            ) {
                state.archive_tasks.update(id, |record| {
                    record.uncertain = Some(UncertainArtifact::New(Box::new(published)));

                    Ok(())
                })?;

                return Err(AppError::CommitUncertain);
            }

            let committed = result.is_ok();

            let worker = Arc::clone(&state);

            state.archive_tasks.update(id, |record| {
                record.status = ArchiveTask::Cleaning;

                Ok(())
            })?;

            let cleaned = tokio::task::spawn_blocking(move || {
                import::cleanup(&published, committed, &worker.resource)
            })
            .await
            .map_err(|_| AppError::Storage)?;

            result?;

            let warnings = match cleaned {
                Ok(()) => Vec::new(),
                Err(_) => vec!["项目已导入，部分临时文件待下次启动清理".to_owned()],
            };

            Ok(ArchiveCompletion { comic_id, warnings })
        }
        ImportArtifact::Existing {
            document,
            comic_id,
            baseline,
            ..
        } => {
            let mode = match mode {
                ArchiveImportMode::FillEmpty => ImportMode::FillEmpty,
                ArchiveImportMode::ReplaceAll => ImportMode::ReplaceAll,
            };

            let input = import::existing(&document, comic_id.clone(), baseline, mode)?;

            let worker = Arc::clone(&state);

            let protected = input.clone();

            let originals = tokio::task::spawn_blocking(move || {
                let comic_id = uuid::Uuid::parse_str(&protected.comic_id)
                    .map_err(|_| AppError::InvalidInput)?;

                protected
                    .baseline
                    .iter()
                    .map(|page| {
                        worker
                            .resource
                            .open_original(
                                &page.image_reference,
                                comic_id,
                                uuid::Uuid::parse_str(&page.id)
                                    .map_err(|_| AppError::InvalidInput)?,
                            )
                            .map_err(|_| AppError::Storage)
                    })
                    .collect::<AppResult<Vec<_>>>()
            })
            .await
            .map_err(|_| AppError::Storage)??;

            let result =
                crate::usecase::import::apply_comic_import(state.database().await?, input.clone())
                    .await;

            drop(originals);

            if matches!(
                result,
                Err(AppError::CommitUncertain | AppError::RecoveryRequired)
            ) {
                state.archive_tasks.update(id, |record| {
                    record.uncertain = Some(UncertainArtifact::Existing(input));

                    Ok(())
                })?;

                return Err(AppError::CommitUncertain);
            }

            result?;

            Ok(ArchiveCompletion {
                comic_id,
                warnings: Vec::new(),
            })
        }
    }
}

/// # Errors
/// Consumes a preview once; the outer write ledger owns replay protection.
pub async fn run(
    state: Arc<Harness>,
    id: String,
    metadata: ComicMetadata,
    mode: ArchiveImportMode,
) -> AppResult<ArchiveCompletion> {
    state.database().await?;

    let artifact = state.archive_tasks.update(&id, |record| {
        if !matches!(record.status, ArchiveTask::AwaitingConfirmation { .. })
            || record.cancel.load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(AppError::Conflict);
        }

        if matches!(record.artifact, Some(ImportArtifact::New(_)))
            && metadata.title.trim().is_empty()
        {
            return Err(AppError::InvalidInput);
        }

        let artifact = record.artifact.take().ok_or(AppError::Conflict)?;

        record.status = ArchiveTask::Committing;

        Ok(artifact)
    })?;

    let result = apply(Arc::clone(&state), &id, artifact, metadata, mode).await;

    state.archive_tasks.update(&id, |record| {
        record.status = match &result {
            Ok(result) => ArchiveTask::Completed {
                result: result.clone(),
            },
            Err(error) => ArchiveTask::Failed {
                message: error.to_string(),
                commit_uncertain: matches!(
                    error,
                    AppError::CommitUncertain | AppError::RecoveryRequired
                ),
            },
        };

        if !matches!(
            result,
            Err(AppError::CommitUncertain | AppError::RecoveryRequired)
        ) {
            record.permit = None;
        }

        Ok(())
    })?;

    result
}
