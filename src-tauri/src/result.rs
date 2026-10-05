use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AppError {
    InvalidInput,
    NotFound,
    Conflict,
    Busy,
    Storage,
    Migration,
    CommitUncertain,
    RecoveryRequired,
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidInput => "输入无效，请检查后重试",
            Self::NotFound => "内容已不存在，请刷新",
            Self::Conflict => "内容已变化，请保留草稿并重新载入",
            Self::Busy => "数据正在使用中，请稍后重试",
            Self::Storage => "无法读写本地数据，请检查磁盘空间和权限",
            Self::Migration => "数据升级失败，原有数据已保留，请修复后重新启动",
            Self::CommitUncertain => "写入结果尚未确认，请等待核实",
            Self::RecoveryRequired => "本地数据需要恢复，请保留数据并重新启动",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(database) => {
                let code = database.code();

                match code.as_deref() {
                    Some("5" | "6" | "261" | "262" | "517" | "773") => Self::Busy,
                    _ if database.is_unique_violation() => Self::Conflict,
                    _ => Self::Storage,
                }
            }
            _ => Self::Storage,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
