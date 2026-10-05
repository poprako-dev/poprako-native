use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Comic {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub author: String,
    pub created_at: i64,
    pub updated_at: i64,
}
