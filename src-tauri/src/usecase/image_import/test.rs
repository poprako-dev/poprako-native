use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use poprako_orchestra::Run;
use tempfile::TempDir;
use tokio::sync::Notify;

use crate::data::comic::ComicMetadata;
use crate::data::image_import::{ImageImportTarget, ImageTaskPhase};
use crate::harness::Harness;
use crate::part::image_selection::{ImageSelectionKind, SelectImages};
use crate::result::{AppError, AppResult};
use crate::usecase::{comic, image_import};

struct Selection {
    output: AppResult<Option<Vec<PathBuf>>>,
    gate: Option<Arc<Notify>>,
}

impl Run<SelectImages> for Selection {
    type Error = AppError;

    async fn run(&self, _: &SelectImages) -> AppResult<Option<Vec<PathBuf>>> {
        if let Some(gate) = &self.gate {
            gate.notified().await;
        }

        self.output.clone()
    }
}

async fn setup() -> Result<(TempDir, Arc<Harness>, ImageImportTarget), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;

    let harness = Arc::new(Harness::open(directory.path().join("managed"))?);

    let comic = comic::create_comic(
        harness.database().await?,
        ComicMetadata {
            title: "Selection workflow".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
    )
    .await?;

    Ok((
        directory,
        harness,
        ImageImportTarget::Append {
            comic_id: comic.id,
            baseline: Vec::new(),
        },
    ))
}

async fn wait(harness: &Harness, task_id: &str) -> AppResult<ImageTaskPhase> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let status = image_import::get_status(harness, task_id)?;

            if !matches!(
                status.phase,
                ImageTaskPhase::Selecting | ImageTaskPhase::Preparing | ImageTaskPhase::Stopping
            ) && harness.resource_task.available_permits() == 1
            {
                return Ok(status.phase);
            }

            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .map_err(|_| AppError::Busy)?
}

#[tokio::test]
async fn selection_cancel_empty_and_failure_release_permit() -> Result<(), Box<dyn Error>> {
    let (_directory, harness, target) = setup().await?;

    for (output, expected) in [
        (Ok(None), ImageTaskPhase::Cancelled),
        (Ok(Some(Vec::new())), ImageTaskPhase::Failed),
        (Err(AppError::Storage), ImageTaskPhase::Failed),
    ] {
        let status = image_import::select(
            Arc::clone(&harness),
            Selection { output, gate: None },
            target.clone(),
            ImageSelectionKind::Folder,
        )
        .await?;

        assert_eq!(status.phase, ImageTaskPhase::Selecting);

        assert_eq!(wait(&harness, &status.task_id).await?, expected);

        assert!(!image_import::get_status(&harness, &status.task_id)?.cleanup_pending);
    }

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn cancellation_during_selection_prevents_preparation() -> Result<(), Box<dyn Error>> {
    let (_directory, harness, target) = setup().await?;

    let gate = Arc::new(Notify::new());

    let status = image_import::select(
        Arc::clone(&harness),
        Selection {
            output: Ok(Some(vec![PathBuf::from("unreadable.png")])),
            gate: Some(Arc::clone(&gate)),
        },
        target,
        ImageSelectionKind::Files,
    )
    .await?;

    assert_eq!(
        image_import::cancel(Arc::clone(&harness), status.task_id.clone())
            .await?
            .phase,
        ImageTaskPhase::Stopping
    );

    assert_eq!(harness.resource_task.available_permits(), 0);

    gate.notify_one();

    assert_eq!(
        wait(&harness, &status.task_id).await?,
        ImageTaskPhase::Cancelled
    );

    let final_status = image_import::get_status(&harness, &status.task_id)?;

    assert_eq!(final_status.total_files, 0);

    assert!(final_status.previews.is_empty());

    harness.database().await?.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn selected_bad_supported_image_fails_explicitly() -> Result<(), Box<dyn Error>> {
    let (directory, harness, target) = setup().await?;

    let comic_id = target.comic_id().to_owned();

    let source = directory.path().join("broken.png");

    std::fs::write(&source, b"not an image")?;

    let status = image_import::select(
        Arc::clone(&harness),
        Selection {
            output: Ok(Some(vec![source])),
            gate: None,
        },
        target,
        ImageSelectionKind::Files,
    )
    .await?;

    assert_eq!(
        wait(&harness, &status.task_id).await?,
        ImageTaskPhase::Failed
    );

    assert!(
        !image_import::get_status(&harness, &status.task_id)?
            .message
            .is_empty()
    );

    assert!(
        comic::get_comic_detail(harness.database().await?, &comic_id)
            .await?
            .pages
            .is_empty()
    );

    harness.database().await?.pool().close().await;

    Ok(())
}
