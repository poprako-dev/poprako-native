use std::error::Error;

use tempfile::tempdir;

use crate::data::comic::{ComicMetadata, MoveChapter, NewComic};
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{
    TermMetadata, TermbaseMetadata, TerminologyContext, TerminologyQuery,
};
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::model::terminology::{Term, Termbase};
use crate::result::AppError;
use crate::usecase::{chapter, comic, terminology};

fn comic_input(title: &str) -> NewComic {
    NewComic {
        comic: ComicMetadata {
            title: title.to_owned(),
            author: String::new(),
        },
        chapter_title: "第一章".to_owned(),
    }
}

pub fn query(text: &str) -> TerminologyQuery {
    TerminologyQuery {
        text: text.to_owned(),
        offset: 0,
        limit: 30,
    }
}

pub async fn base(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    name: &str,
) -> Result<Termbase, Box<dyn Error>> {
    Ok(terminology::create_termbase(
        coordinator,
        access,
        TermbaseMetadata {
            name: name.to_owned(),
            description: "说明".to_owned(),
        },
        evidence_handle(),
    )
    .await?)
}

pub fn metadata(source: &str) -> TermMetadata {
    TermMetadata {
        source: source.to_owned(),
        targets: vec!["首选".to_owned(), "次选".to_owned()],
        comment: "备注".to_owned(),
    }
}

pub async fn term(
    coordinator: &SqliteCoordinator,
    access: TerminologyContext,
    base: &Termbase,
    source: &str,
) -> Result<Term, Box<dyn Error>> {
    Ok(terminology::create_term(
        coordinator,
        access,
        base.id.clone(),
        metadata(source),
        evidence_handle(),
    )
    .await?)
}

async fn assert_global_readonly(
    coordinator: &SqliteCoordinator,
    access: &TerminologyContext,
    global: &Termbase,
    global_entry: Term,
) {
    assert_eq!(
        terminology::create_term(
            coordinator,
            access.clone(),
            global.id.clone(),
            metadata("他の語"),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );

    assert_eq!(
        terminology::update_term(
            coordinator,
            access.clone(),
            global_entry.clone(),
            metadata("other"),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );

    assert_eq!(
        terminology::delete_term(coordinator, access.clone(), global_entry, evidence_handle())
            .await,
        Err(AppError::NotFound)
    );

    assert_eq!(
        terminology::delete_termbase(
            coordinator,
            access.clone(),
            global.clone(),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );
}

async fn assert_persisted_global(
    coordinator: &SqliteCoordinator,
    id: String,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        terminology::list_term_infos(coordinator, TerminologyContext::Global, id, query("elf"))
            .await?
            .len(),
        1
    );

    Ok(())
}

#[tokio::test]
async fn scopes_follow_current_chapter_parent_and_global_libraries_are_readonly_in_translator()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let database_path = directory.path().join("test.sqlite");

    let coordinator = SqliteCoordinator::new(database::open(&database_path).await?);

    let source = comic::create_comic(&coordinator, comic_input("源漫画")).await?;

    let second = chapter::create_chapter(
        &coordinator,
        &source.comic_id,
        vec![source.id.clone()],
        crate::data::chapter::ChapterMetadata {
            title: "第二章".to_owned(),
        },
    )
    .await?;

    let target = comic::create_comic(&coordinator, comic_input("目标漫画")).await?;

    let access = TerminologyContext::Chapter {
        chapter_id: source.id.clone(),
    };

    let other = TerminologyContext::Chapter {
        chapter_id: target.id.clone(),
    };

    let shared = TerminologyContext::Chapter {
        chapter_id: second.id.clone(),
    };

    let global = base(&coordinator, TerminologyContext::Global, "全局").await?;

    let local = base(&coordinator, access.clone(), "本作").await?;

    let destination = base(&coordinator, other.clone(), "本作").await?;

    let entry = term(&coordinator, access.clone(), &local, "団長").await?;

    let global_entry = term(&coordinator, TerminologyContext::Global, &global, "Elf").await?;

    assert_eq!(
        terminology::list_term_infos(&coordinator, shared.clone(), local.id.clone(), query(""))
            .await?,
        vec![entry.clone()]
    );

    assert_eq!(
        terminology::list_termbase_infos(&coordinator, access.clone(), query(""))
            .await?
            .len(),
        2
    );

    assert_eq!(
        terminology::get_termbase(&coordinator, other.clone(), local.id.clone()).await,
        Err(AppError::NotFound)
    );

    assert_global_readonly(&coordinator, &access, &global, global_entry).await;

    comic::move_chapter(
        &coordinator,
        MoveChapter {
            chapter_id: source.id.clone(),
            source_comic_id: source.comic_id.clone(),
            target_comic_id: target.comic_id,
            source_baseline: vec![source.id.clone(), second.id.clone()],
            target_baseline: vec![target.id],
        },
    )
    .await?;

    assert!(
        terminology::get_termbase(&coordinator, access.clone(), destination.id)
            .await
            .is_ok()
    );

    assert_eq!(
        terminology::get_termbase(&coordinator, access, local.id.clone()).await,
        Err(AppError::NotFound)
    );

    chapter::delete_chapter(&coordinator, &second.id).await?;

    let existing = crate::implementation::repository::terminology::read::base(
        &mut *coordinator.pool().acquire().await?,
        &local.id,
    )
    .await?;

    assert_eq!(existing.id, local.id);

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        comic::delete_comic(&coordinator, &source.comic_id).await,
        Err(AppError::Storage)
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    comic::delete_comic(&coordinator, &source.comic_id).await?;

    assert_eq!(
        crate::implementation::repository::terminology::read::term(
            &mut *coordinator.pool().acquire().await?,
            &entry.id
        )
        .await,
        Err(AppError::NotFound)
    );

    assert!(
        terminology::get_termbase(&coordinator, TerminologyContext::Global, global.id.clone())
            .await
            .is_ok()
    );

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&database_path).await?);

    assert_persisted_global(&reopened, global.id).await?;

    reopened.pool().close().await;

    Ok(())
}
