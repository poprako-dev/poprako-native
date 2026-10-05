use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tokio::sync::watch;
use uuid::Uuid;

use crate::result::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WriteRequest {
    pub session_id: String,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct WriteSession {
    pub session_id: String,
    pub next_sequence: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WriteStatus {
    NotAccepted,
    Pending,
    Committed,
    NotCommitted { error: AppError },
    Uncertain,
    Consumed,
}

pub type Confirmation =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = AppResult<Value>> + Send>> + Send + Sync>;

struct Record {
    confirmation: Option<Confirmation>,
    sequence: u32,
    payload: Value,
    result: Option<AppResult<Value>>,
}

struct LedgerState {
    session_id: String,
    consumed: u32,
    closing: bool,
    record: Option<Record>,
}

pub struct WriteLedger {
    state: Mutex<LedgerState>,
    send: watch::Sender<()>,
}

impl Default for WriteLedger {
    fn default() -> Self {
        let (send, _) = watch::channel(());

        Self {
            state: Mutex::new(LedgerState {
                session_id: Uuid::new_v4().to_string(),
                consumed: 0,
                closing: false,
                record: None,
            }),
            send,
        }
    }
}

impl WriteLedger {
    /// # Errors
    /// Returns a recovery error if state cannot be inspected safely.
    pub fn session(&self) -> AppResult<WriteSession> {
        let state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        Ok(WriteSession {
            session_id: state.session_id.clone(),
            next_sequence: state
                .consumed
                .checked_add(1)
                .ok_or(AppError::RecoveryRequired)?,
        })
    }

    fn validate(state: &LedgerState, request: &WriteRequest) -> AppResult<()> {
        if request.session_id != state.session_id || request.sequence == 0 {
            return Err(AppError::InvalidInput);
        }

        Ok(())
    }

    fn begin(
        &self,
        request: &WriteRequest,
        payload: Value,
        confirmation: Option<Confirmation>,
    ) -> AppResult<bool> {
        let mut state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        Self::validate(&state, request)?;

        if state.closing {
            return Err(AppError::Busy);
        }

        if request.sequence <= state.consumed {
            return Err(AppError::Conflict);
        }

        if let Some(record) = &state.record {
            if request.sequence == record.sequence && payload == record.payload {
                return Ok(false);
            }

            return Err(AppError::Conflict);
        }

        if Some(request.sequence) != state.consumed.checked_add(1) {
            return Err(AppError::InvalidInput);
        }

        state.record = Some(Record {
            sequence: request.sequence,
            payload,
            result: None,
            confirmation,
        });

        Ok(true)
    }

    fn complete(&self, request: &WriteRequest, result: AppResult<Value>) -> AppResult<()> {
        let mut state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        let record = state.record.as_mut().ok_or(AppError::RecoveryRequired)?;

        if record.sequence != request.sequence {
            return Err(AppError::RecoveryRequired);
        }

        record.result = Some(result);

        self.send.send_replace(());

        Ok(())
    }

    fn result(&self, request: &WriteRequest) -> AppResult<Option<AppResult<Value>>> {
        let state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        Self::validate(&state, request)?;

        let record = state.record.as_ref().ok_or(AppError::NotFound)?;

        if record.sequence != request.sequence {
            return Err(AppError::Conflict);
        }

        Ok(record.result.clone())
    }

    /// Starts verification of retained evidence without re-running the mutation.
    /// # Errors
    /// Invalid request ownership or poisoned state requires recovery.
    pub fn refresh(self: &Arc<Self>, request: &WriteRequest) -> AppResult<()> {
        let confirmation = {
            let mut state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

            Self::validate(&state, request)?;

            let Some(record) = state
                .record
                .as_mut()
                .filter(|record| record.sequence == request.sequence)
            else {
                return Ok(());
            };

            if !matches!(
                record.result,
                Some(Err(AppError::CommitUncertain | AppError::RecoveryRequired))
            ) {
                return Ok(());
            }

            let Some(confirmation) = record.confirmation.clone() else {
                return Ok(());
            };

            record.result = None;

            confirmation
        };

        let worker = Arc::clone(self);

        let request = request.clone();

        tokio::spawn(async move {
            let execution = tokio::spawn(async move { confirmation().await });

            let result = execution.await.unwrap_or(Err(AppError::CommitUncertain));

            if worker.complete(&request, result).is_err() {
                eprintln!("Unable to record write verification; recovery is required");
            }
        });

        Ok(())
    }

    /// # Errors
    /// Invalid sessions cannot inspect or consume another window's writes.
    pub fn status(&self, request: &WriteRequest) -> AppResult<WriteStatus> {
        let state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        Self::validate(&state, request)?;

        if request.sequence <= state.consumed {
            return Ok(WriteStatus::Consumed);
        }

        let Some(record) = state
            .record
            .as_ref()
            .filter(|record| record.sequence == request.sequence)
        else {
            return Ok(WriteStatus::NotAccepted);
        };

        Ok(match &record.result {
            None => WriteStatus::Pending,
            Some(Ok(_)) => WriteStatus::Committed,
            Some(Err(AppError::CommitUncertain | AppError::RecoveryRequired)) => {
                WriteStatus::Uncertain
            }
            Some(Err(error)) => WriteStatus::NotCommitted {
                error: error.clone(),
            },
        })
    }

    /// # Errors
    /// Pending or uncertain writes cannot be acknowledged and forgotten.
    pub fn acknowledge(&self, request: &WriteRequest) -> AppResult<()> {
        match self.status(request)? {
            WriteStatus::Consumed => return Ok(()),
            WriteStatus::Committed | WriteStatus::NotCommitted { .. } => {}
            _ => return Err(AppError::Busy),
        }

        let mut state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        if state
            .record
            .as_ref()
            .is_some_and(|record| record.sequence == request.sequence)
        {
            state.consumed = request.sequence;

            state.record = None;
        }

        Ok(())
    }

    /// # Errors
    /// Closing while an accepted write is pending would lose its result.
    pub fn begin_close(&self) -> AppResult<()> {
        let mut state = self.state.lock().map_err(|_| AppError::RecoveryRequired)?;

        if state.record.as_ref().is_some_and(|record| {
            matches!(
                record.result,
                None | Some(Err(AppError::CommitUncertain | AppError::RecoveryRequired))
            )
        }) {
            return Err(AppError::Busy);
        }

        state.closing = true;

        Ok(())
    }
}

/// # Errors
/// Returns the original typed outcome without ever re-executing an accepted request.
pub async fn execute_with_confirmation<T, F, Fut>(
    ledger: Arc<WriteLedger>,
    request: WriteRequest,
    payload: Value,
    confirmation: Option<Confirmation>,
    operation: F,
) -> AppResult<T>
where
    T: Serialize + DeserializeOwned + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = AppResult<T>> + Send + 'static,
{
    let mut recv = ledger.send.subscribe();

    if ledger.begin(&request, payload, confirmation)? {
        let worker = Arc::clone(&ledger);

        let worker_request = request.clone();

        // The accepted operation outlives cancellation of the IPC response future.
        tokio::spawn(async move {
            let execution = tokio::spawn(async move { operation().await });

            let result = match execution.await {
                Ok(result) => result.and_then(|value| {
                    serde_json::to_value(value).map_err(|_| AppError::CommitUncertain)
                }),
                Err(_) => Err(AppError::CommitUncertain),
            };

            if worker.complete(&worker_request, result).is_err() {
                eprintln!("Unable to record the accepted write outcome; recovery is required");
            }
        });
    }

    ledger.refresh(&request)?;

    loop {
        if let Some(result) = ledger.result(&request)? {
            return serde_json::from_value(result?).map_err(|_| AppError::RecoveryRequired);
        }

        recv.changed()
            .await
            .map_err(|_| AppError::RecoveryRequired)?;
    }
}

/// # Errors
/// Returns the cached outcome for retries of an identical accepted operation.
pub async fn execute<T, F, Fut>(
    ledger: Arc<WriteLedger>,
    request: WriteRequest,
    payload: Value,
    operation: F,
) -> AppResult<T>
where
    T: Serialize + DeserializeOwned + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = AppResult<T>> + Send + 'static,
{
    execute_with_confirmation(ledger, request, payload, None, operation).await
}

#[cfg(test)]
mod test;
