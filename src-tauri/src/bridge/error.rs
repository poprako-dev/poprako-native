use serde::Serialize;
use specta::Type;

use crate::result::AppError;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Recovery {
    Retry,
    Reload,
    CorrectInput,
    WaitForConfirmation,
    Restart,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub recovery: Recovery,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        let (code, recovery) = match error {
            AppError::InvalidInput => ("validation", Recovery::CorrectInput),
            AppError::NotFound => ("not_found", Recovery::Reload),
            AppError::Conflict => ("conflict", Recovery::Reload),
            AppError::Busy => ("storage_busy", Recovery::Retry),
            AppError::Storage => ("storage", Recovery::Retry),
            AppError::Migration => ("migration", Recovery::Restart),
            AppError::CommitUncertain => ("commit_unknown", Recovery::WaitForConfirmation),
            AppError::RecoveryRequired => ("recovery_required", Recovery::Restart),
        };

        Self {
            code: code.to_owned(),
            message: error.to_string(),
            recovery,
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
