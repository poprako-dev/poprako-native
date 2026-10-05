use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;
use tokio::sync::Notify;

use crate::implementation::write::{WriteLedger, WriteRequest, WriteStatus, execute};
use crate::result::{AppError, AppResult};

fn request(ledger: &WriteLedger) -> AppResult<WriteRequest> {
    let session = ledger.session()?;

    Ok(WriteRequest {
        session_id: session.session_id,
        sequence: session.next_sequence,
    })
}

#[tokio::test]
async fn concurrent_duplicate_executes_once_and_consumed_request_cannot_replay() -> AppResult<()> {
    let ledger = Arc::new(WriteLedger::default());

    let input = request(&ledger)?;

    let count = AtomicUsize::new(0);

    let first_count = Arc::new(count);

    let second_count = Arc::clone(&first_count);

    let first = Arc::clone(&first_count);

    let (a, b) = tokio::join!(
        execute(
            Arc::clone(&ledger),
            input.clone(),
            json!(["save", 1]),
            move || async move {
                first.fetch_add(1, Ordering::SeqCst);

                Ok(42)
            }
        ),
        execute(
            Arc::clone(&ledger),
            input.clone(),
            json!(["save", 1]),
            move || async move {
                second_count.fetch_add(1, Ordering::SeqCst);

                Ok(99)
            }
        )
    );

    assert_eq!(a?, 42);

    assert_eq!(b?, 42);

    assert_eq!(first_count.load(Ordering::SeqCst), 1);

    ledger.acknowledge(&input)?;

    ledger.acknowledge(&input)?;

    assert!(matches!(ledger.status(&input)?, WriteStatus::Consumed));

    let replay = execute(ledger, input, json!(["save", 1]), || async { Ok(1) }).await;

    assert_eq!(replay, Err(AppError::Conflict));

    Ok(())
}

#[tokio::test]
async fn cancelled_response_does_not_cancel_accepted_operation() -> AppResult<()> {
    let ledger = Arc::new(WriteLedger::default());

    let input = request(&ledger)?;

    let started = Arc::new(Notify::new());

    let finish = Arc::new(Notify::new());

    let accepted = Arc::clone(&started);

    let completed = Arc::clone(&finish);

    let worker_ledger = Arc::clone(&ledger);

    let worker_request = input.clone();

    let response = tokio::spawn(async move {
        execute(
            worker_ledger,
            worker_request,
            json!("fixed"),
            move || async move {
                accepted.notify_one();

                completed.notified().await;

                Ok(7)
            },
        )
        .await
    });

    started.notified().await;

    response.abort();

    assert_eq!(ledger.begin_close(), Err(AppError::Busy));

    finish.notify_one();

    let result = execute(Arc::clone(&ledger), input, json!("fixed"), || async {
        Ok(8)
    })
    .await?;

    assert_eq!(result, 7);

    Ok(())
}

#[tokio::test]
async fn same_request_with_different_payload_is_rejected() -> AppResult<()> {
    let ledger = Arc::new(WriteLedger::default());

    let input = request(&ledger)?;

    execute(Arc::clone(&ledger), input.clone(), json!(1), || async {
        Ok(5)
    })
    .await?;

    let mismatch = execute(ledger, input, json!(2), || async { Ok(6) }).await;

    assert_eq!(mismatch, Err(AppError::Conflict));

    Ok(())
}

#[tokio::test]
async fn entering_close_atomically_rejects_new_writes() -> AppResult<()> {
    let ledger = Arc::new(WriteLedger::default());

    let input = request(&ledger)?;

    ledger.begin_close()?;

    let result = execute(ledger, input, json!(1), || async { Ok(5) }).await;

    assert_eq!(result, Err(AppError::Busy));

    Ok(())
}

#[tokio::test]
async fn unknown_outcome_is_verified_without_reexecuting_mutation() -> AppResult<()> {
    use crate::implementation::write::{Confirmation, execute_with_confirmation};

    let ledger = Arc::new(WriteLedger::default());

    let input = request(&ledger)?;

    let confirmations = Arc::new(AtomicUsize::new(0));

    let count = Arc::clone(&confirmations);

    let confirmation: Confirmation = Arc::new(move || {
        let count = Arc::clone(&count);

        Box::pin(async move {
            match count.fetch_add(1, Ordering::SeqCst) {
                0 => Err(AppError::CommitUncertain),
                _ => Ok(json!(42)),
            }
        })
    });

    let first: AppResult<i32> = execute_with_confirmation(
        Arc::clone(&ledger),
        input.clone(),
        json!(1),
        Some(confirmation),
        || async { Err(AppError::CommitUncertain) },
    )
    .await;

    assert_eq!(first, Err(AppError::CommitUncertain));

    let mut recovered = Err(AppError::CommitUncertain);

    for _ in 0..2 {
        recovered = execute(Arc::clone(&ledger), input.clone(), json!(1), || async {
            Err::<i32, _>(AppError::InvalidInput)
        })
        .await;

        if recovered.is_ok() {
            break;
        }
    }

    assert_eq!(recovered?, 42);

    assert_eq!(confirmations.load(Ordering::SeqCst), 2);

    ledger.acknowledge(&input)?;

    Ok(())
}
