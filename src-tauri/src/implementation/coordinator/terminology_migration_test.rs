use std::error::Error;
use std::fs;

use tempfile::tempdir;

use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermMetadata, TermbaseMetadata, TerminologyContext};
use crate::implementation::coordinator::hierarchy_migration_test::{connect, legacy};
use crate::implementation::coordinator::{SqliteCoordinator, database};
use crate::usecase::{chapter, editor, terminology};

const CHAPTER: &str = "11111111-1111-4111-8111-111111111111";
const PAGE: &str = "33333333-3333-4333-8333-333333333333";

async fn populate(coordinator: &SqliteCoordinator) -> Result<(), Box<dyn Error>> {
    for access in [
        TerminologyContext::Global,
        TerminologyContext::Chapter {
            chapter_id: CHAPTER.to_owned(),
        },
    ] {
        let base = terminology::create_termbase(
            coordinator,
            access.clone(),
            TermbaseMetadata {
                name: "测试库".to_owned(),
                description: String::new(),
            },
            evidence_handle(),
        )
        .await?;

        terminology::create_term(
            coordinator,
            access,
            base.id,
            TermMetadata {
                source: "原文".to_owned(),
                targets: vec!["译名".to_owned()],
                comment: String::new(),
            },
            evidence_handle(),
        )
        .await?;
    }

    Ok(())
}

#[tokio::test]
async fn terminology_upgrade_rolls_back_retries_and_preserves_editor_content()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let (path, migration) = legacy(directory.path()).await?;

    fs::write(
        migration.join("202610190001_comic_chapter.sql"),
        include_str!("../../../migration/202610190001_comic_chapter.sql"),
    )?;

    let pool = connect(&path).await?;

    sqlx::migrate::Migrator::new(migration.as_path())
        .await?
        .run(&pool)
        .await?;

    let old = SqliteCoordinator::new(pool);

    let before = editor::get_page_editor(&old, CHAPTER, PAGE).await?;

    let detail = chapter::get_chapter_detail(&old, CHAPTER).await?;

    old.pool().close().await;

    let backup = directory.path().join("backup.sqlite");

    fs::copy(&path, &backup)?;

    fs::write(
        migration.join("202610210001_terminology.sql"),
        format!(
            "{}\nCREATE TABLE failed (value INTEGER CHECK(value = 1)); INSERT INTO failed VALUES (0);",
            include_str!("../../../migration/202610210001_terminology.sql")
        ),
    )?;

    let pool = connect(&path).await?;

    assert!(
        sqlx::migrate::Migrator::new(migration.as_path())
            .await?
            .run(&pool)
            .await
            .is_err()
    );

    let tables = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sqlite_schema WHERE name IN ('termbase', 'term')"
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(tables, 0);

    let failed = SqliteCoordinator::new(pool);

    assert_eq!(
        editor::get_page_editor(&failed, CHAPTER, PAGE).await?,
        before
    );

    failed.pool().close().await;

    let migrated = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        chapter::get_chapter_detail(&migrated, CHAPTER).await?,
        detail
    );

    populate(&migrated).await?;

    assert_eq!(
        editor::get_page_editor(&migrated, CHAPTER, PAGE).await?,
        before
    );

    let after = chapter::get_chapter_detail(&migrated, CHAPTER).await?;

    assert_eq!(after.chapter, detail.chapter);

    assert_eq!(after.work_position, detail.work_position);

    assert_ne!(after.comic.updated_at, detail.comic.updated_at);

    migrated.pool().close().await;

    fs::copy(&backup, &path)?;

    let restored = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        editor::get_page_editor(&restored, CHAPTER, PAGE).await?,
        before
    );

    restored.pool().close().await;

    Ok(())
}
