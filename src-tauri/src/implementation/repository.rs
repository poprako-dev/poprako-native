pub mod chapter_order;
pub mod comic;
pub mod library;
use sqlx::SqlitePool;

pub mod catalog;
pub mod chapter;
pub mod editor;
pub mod export;
pub mod import;
pub mod page;
pub mod page_order;
pub mod preference;
pub mod read;
pub mod recovery;
pub mod search;
pub mod work_position;

#[cfg(test)]
mod behavior_test;
#[cfg(test)]
mod import_test;
#[cfg(test)]
mod recovery_test;
#[cfg(test)]
mod search_test;
#[cfg(test)]
mod statistic_test;

pub struct SqliteRepository {
    pub pool: SqlitePool,
}

impl SqliteRepository {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod hierarchy_test;

#[cfg(test)]
mod chapter_move_test;

#[cfg(test)]
mod undo_test;
