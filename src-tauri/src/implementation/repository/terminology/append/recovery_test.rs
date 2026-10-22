use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tempfile::tempdir;

use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermbaseAppend, TermbaseAppendResult, TerminologyContext};
use crate::implementation::coordinator::{SqliteCoordinator, database};
use crate::implementation::repository::terminology::test::{base, metadata, query, term};
use crate::implementation::write::{
    Confirmation, WriteLedger, WriteRequest, execute, execute_with_confirmation,
};
use crate::result::AppError;
use crate::usecase::recovery::verify_evidence;
use crate::usecase::terminology;
use crate::usecase::terminology::append::append_termbase;

#[tokio::test]
async fn repeated_unknown_request_verifies_the_append_without_reappending_comments()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator = Arc::new(SqliteCoordinator::new(
        database::open(&directory.path().join("test.sqlite")).await?,
    ));

    let access = TerminologyContext::Global;

    let library = base(&coordinator, access.clone(), "Library").await?;

    term(&coordinator, access.clone(), &library, "Elf").await?;

    let ledger = Arc::new(WriteLedger::default());

    let session = ledger.session()?;

    let request = WriteRequest {
        session_id: session.session_id,
        sequence: session.next_sequence,
    };

    let evidence = evidence_handle();

    let retained = Arc::clone(&evidence);

    let database = Arc::clone(&coordinator);

    let confirmation: Confirmation = Arc::new(move || {
        let retained = Arc::clone(&retained);

        let database = Arc::clone(&database);

        Box::pin(async move {
            let evidence = retained
                .lock()
                .map_err(|_| AppError::RecoveryRequired)?
                .clone()
                .ok_or(AppError::CommitUncertain)?;

            let result = verify_evidence(&database, &evidence).await?;

            serde_json::to_value(result).map_err(|_| AppError::RecoveryRequired)
        })
    });

    let mutations = Arc::new(AtomicUsize::new(0));

    let count = Arc::clone(&mutations);

    let database = Arc::clone(&coordinator);

    let base_id = library.id.clone();

    let input = TermbaseAppend {
        terms: vec![crate::data::terminology::TermMetadata {
            comment: "New note".to_owned(),
            ..metadata("Elf")
        }],
    };

    let payload = serde_json::json!(["append_termbase", access, base_id, input]);

    let first: Result<TermbaseAppendResult, AppError> = execute_with_confirmation(
        Arc::clone(&ledger),
        request.clone(),
        payload.clone(),
        Some(confirmation),
        move || async move {
            count.fetch_add(1, Ordering::SeqCst);

            append_termbase(&database, access, base_id, input, evidence).await?;

            Err(AppError::CommitUncertain)
        },
    )
    .await;

    assert_eq!(first, Err(AppError::CommitUncertain));

    for _ in 0..2 {
        let count = Arc::clone(&mutations);

        let result: Result<TermbaseAppendResult, AppError> = execute(
            Arc::clone(&ledger),
            request.clone(),
            payload.clone(),
            move || async move {
                count.fetch_add(1, Ordering::SeqCst);

                Err(AppError::InvalidInput)
            },
        )
        .await;

        assert_eq!(
            result?,
            TermbaseAppendResult {
                updated: 1,
                ..TermbaseAppendResult::default()
            }
        );
    }

    assert_eq!(mutations.load(Ordering::SeqCst), 1);

    let terms = terminology::list_term_infos(
        &coordinator,
        TerminologyContext::Global,
        library.id,
        query(""),
    )
    .await?;

    assert_eq!(terms[0].comment, "备注\nNew note");

    ledger.acknowledge(&request)?;

    coordinator.pool().close().await;

    Ok(())
}
