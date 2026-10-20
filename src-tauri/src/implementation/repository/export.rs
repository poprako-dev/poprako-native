use poprako_orchestra::Run;

use crate::data::export::ExportSnapshot;
use crate::implementation::repository::SqliteRepository;
use crate::implementation::repository::catalog::read_chapter;
use crate::implementation::repository::read;
use crate::part::repository::export::ReadExportSnapshot;
use crate::result::{AppError, AppResult};

impl Run<ReadExportSnapshot> for SqliteRepository {
    type Error = AppError;

    async fn run(&self, operation: &ReadExportSnapshot) -> AppResult<ExportSnapshot> {
        let mut transaction = self.pool.begin().await?;

        let chapter = read_chapter(&mut transaction, &operation.chapter_id).await?;

        let rows = sqlx::query!(
            "SELECT id FROM page WHERE chapter_id = ? ORDER BY position",
            operation.chapter_id
        )
        .fetch_all(&mut *transaction)
        .await?;

        let mut pages = Vec::with_capacity(rows.len());

        for row in rows {
            pages.push(read::editor(&mut transaction, &operation.chapter_id, &row.id).await?);
        }

        let comic = crate::implementation::repository::library::read_comic(
            &mut transaction,
            &chapter.comic_id,
        )
        .await?;

        transaction.commit().await?;

        Ok(ExportSnapshot {
            comic,
            chapter,
            pages,
        })
    }
}
