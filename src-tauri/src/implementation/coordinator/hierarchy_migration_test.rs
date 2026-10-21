use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tempfile::tempdir;
use uuid::Uuid;

use crate::implementation::coordinator::{SqliteCoordinator, database};
use crate::implementation::resource::ResourceStore;
use crate::model::work_position::EditorMode;
use crate::usecase::{chapter, comic, editor};

const CHAPTER: &str = "11111111-1111-4111-8111-111111111111";
const EMPTY: &str = "22222222-2222-4222-8222-222222222222";
const PAGE: &str = "33333333-3333-4333-8333-333333333333";
const UNIT: &str = "44444444-4444-4444-8444-444444444444";
const REFERENCE: &str = "library/11111111-1111-4111-8111-111111111111/33333333-3333-4333-8333-333333333333/55555555-5555-4555-8555-555555555555.png";

pub async fn connect(path: &Path) -> Result<SqlitePool, Box<dyn Error>> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?)
}

pub async fn legacy(root: &Path) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let migration = root.join("migration");

    fs::create_dir(&migration)?;

    for (name, source) in [
        (
            "202609260001_initial.sql",
            include_str!("../../../migration/202609260001_initial.sql"),
        ),
        (
            "202610030001_light_appearance.sql",
            include_str!("../../../migration/202610030001_light_appearance.sql"),
        ),
        (
            "202610040001_web_shortcuts.sql",
            include_str!("../../../migration/202610040001_web_shortcuts.sql"),
        ),
    ] {
        fs::write(migration.join(name), source)?;
    }

    let path = root.join("database.sqlite");

    let pool = connect(&path).await?;

    sqlx::migrate::Migrator::new(migration.as_path())
        .await?
        .run(&pool)
        .await?;

    // This fixture intentionally targets the published schema, not current query metadata.
    sqlx::raw_sql(include_str!("hierarchy_fixture.sql"))
        .execute(&pool)
        .await?;

    pool.close().await;

    let image = root.join(REFERENCE);

    fs::create_dir_all(image.parent().ok_or("missing parent")?)?;

    image::RgbImage::new(2, 3).save(image)?;

    Ok((path, migration))
}

#[tokio::test]
async fn legacy_projects_upgrade_without_merging_or_rewriting_content() -> Result<(), Box<dyn Error>>
{
    let directory = tempdir()?;

    let (path, _) = legacy(directory.path()).await?;

    let original = fs::read(directory.path().join(REFERENCE))?;

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let comics = comic::list_comic_infos(&coordinator, 0, 50).await?;

    assert_eq!(comics.len(), 2);

    assert!(
        comics
            .iter()
            .all(|info| info.comic.title == "同名漫画" && info.chapter_count == 1)
    );

    assert_ne!(comics[0].comic.id, comics[1].comic.id);

    for info in &comics {
        assert_eq!(Uuid::parse_str(&info.comic.id)?.get_version_num(), 4);
    }

    let detail = chapter::get_chapter_detail(&coordinator, CHAPTER).await?;

    assert_eq!(detail.chapter.title, "序章");

    assert_eq!(detail.comic.author, "原作者");

    assert_eq!(
        (detail.chapter.created_at, detail.chapter.updated_at),
        (100, 200)
    );

    assert_eq!(
        (detail.comic.created_at, detail.comic.updated_at),
        (100, 200)
    );

    let position = detail.work_position.ok_or("missing work position")?;

    assert_eq!(position.page_id.as_deref(), Some(PAGE));

    assert_eq!(position.unit_id.as_deref(), Some(UNIT));

    assert_eq!(position.mode, EditorMode::Proofreading);

    assert_eq!(position.last_opened_at, 900);

    let saved = editor::get_page_editor(&coordinator, CHAPTER, PAGE).await?;

    assert_eq!(saved.page.unit_revision, 17);

    assert_eq!(saved.page.image.reference, REFERENCE);

    assert_eq!(saved.page.image.original_name, "原图.png");

    assert_eq!((saved.page.created_at, saved.page.updated_at), (101, 201));

    let unit = saved.units.first().ok_or("missing unit")?;

    assert_eq!(unit.id, UNIT);

    assert_eq!((unit.x_coord, unit.y_coord), (0.25, 0.75));

    assert_eq!(unit.translated_text, "  译文\n原样保留");

    assert_eq!(unit.proofread_text, "校对正文");

    assert!(unit.is_flagged && unit.is_proofread && !unit.is_bubble);

    assert_eq!((unit.created_at, unit.updated_at), (102, 202));

    let empty = chapter::get_chapter_detail(&coordinator, EMPTY).await?;

    assert_eq!(empty.chapter.title, "第1章");

    assert!(empty.pages.is_empty());

    let store = ResourceStore::open(directory.path())?;

    assert_eq!(
        fs::read(store.resolve(REFERENCE, Uuid::parse_str(CHAPTER)?, Uuid::parse_str(PAGE)?)?)?,
        original
    );

    let violations =
        sqlx::query!(r#"SELECT COUNT(*) AS "count!: i64" FROM pragma_foreign_key_check"#)
            .fetch_one(coordinator.pool())
            .await?;

    assert_eq!(violations.count, 0);

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(comic::list_comic_infos(&reopened, 0, 50).await?, comics);

    assert_eq!(
        editor::get_page_editor(&reopened, CHAPTER, PAGE).await?,
        saved
    );

    reopened.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn failed_upgrade_rolls_back_and_closed_database_backup_restores()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let (path, migration) = legacy(directory.path()).await?;

    let backup = directory.path().join("backup.sqlite");

    fs::copy(&path, &backup)?;

    let original = fs::read(directory.path().join(REFERENCE))?;

    let failed = format!(
        "{}\nCREATE TABLE migration_failure (valid INTEGER CHECK(valid = 1)); INSERT INTO migration_failure VALUES (0);",
        include_str!("../../../migration/202610190001_comic_chapter.sql")
    );

    fs::write(migration.join("202610190001_comic_chapter.sql"), failed)?;

    let pool = connect(&path).await?;

    assert!(
        sqlx::migrate::Migrator::new(migration.as_path())
            .await?
            .run(&pool)
            .await
            .is_err()
    );

    let old = sqlx::query!("SELECT id, title, author FROM comic ORDER BY id")
        .fetch_all(&pool)
        .await?;

    assert_eq!(old.len(), 2);

    assert_eq!(old[0].id, CHAPTER);

    assert_eq!(old[0].author, "原作者");

    let chapters = sqlx::query_scalar!("SELECT COUNT(*) FROM sqlite_schema WHERE name = 'chapter'")
        .fetch_one(&pool)
        .await?;

    assert_eq!(chapters, 0);

    assert_eq!(fs::read(directory.path().join(REFERENCE))?, original);

    pool.close().await;

    let upgraded = SqliteCoordinator::new(database::open(&path).await?);

    let saved = editor::get_page_editor(&upgraded, CHAPTER, PAGE).await?;

    upgraded.pool().close().await;

    fs::copy(&backup, &path)?;

    let restored = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        editor::get_page_editor(&restored, CHAPTER, PAGE).await?,
        saved
    );

    restored.pool().close().await;

    Ok(())
}
