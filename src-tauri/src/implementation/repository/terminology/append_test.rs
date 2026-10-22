use std::error::Error;

use tempfile::tempdir;

use crate::data::comic::{ComicMetadata, NewComic};
use crate::data::recovery::{RecoveryScope, evidence_handle};
use crate::data::terminology::{
    TermMetadata, TermbaseAppend, TermbaseAppendResult, TerminologyContext,
};
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::implementation::repository::terminology::test::{base, metadata, query, term};
use crate::model::terminology::Term;
use crate::part::repository::terminology::AppendTermbase;
use crate::result::AppError;
use crate::usecase::recovery::execute;
use crate::usecase::terminology::append::append_termbase;
use crate::usecase::{comic, terminology};

fn document() -> TermbaseAppend {
    TermbaseAppend {
        terms: vec![
            TermMetadata {
                source: " eLF　".to_owned(),
                targets: vec!["　首选　".to_owned(), " new　".to_owned()],
                comment: " new note　".to_owned(),
            },
            metadata(" Dwarf　"),
        ],
    }
}

fn unchanged_document(terms: &[Term]) -> TermbaseAppend {
    TermbaseAppend {
        terms: terms
            .iter()
            .map(|value| TermMetadata {
                source: value.source.clone(),
                targets: value.targets.clone(),
                comment: value.comment.clone(),
            })
            .collect(),
    }
}

#[tokio::test]
async fn mixed_append_preserves_identity_order_metadata_and_survives_commit_loss_and_reopen()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let path = directory.path().join("test.sqlite");

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    let access = TerminologyContext::Global;

    let library = base(&coordinator, access.clone(), "Library").await?;

    let original = term(&coordinator, access.clone(), &library, "Elf").await?;

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        append_termbase(
            &coordinator,
            access.clone(),
            library.id.clone(),
            document(),
            evidence_handle()
        )
        .await,
        Err(AppError::Storage)
    );

    assert_eq!(
        terminology::list_term_infos(&coordinator, access.clone(), library.id.clone(), query(""))
            .await?,
        vec![original.clone()]
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    assert_eq!(
        append_termbase(
            &coordinator,
            access.clone(),
            library.id.clone(),
            document(),
            evidence_handle()
        )
        .await?,
        TermbaseAppendResult {
            added: 1,
            updated: 1,
            unchanged: 0
        }
    );

    let terms =
        terminology::list_term_infos(&coordinator, access.clone(), library.id.clone(), query(""))
            .await?;

    let elf = terms
        .iter()
        .find(|value| value.id == original.id)
        .ok_or("missing original")?;

    assert_eq!(elf.created_at, original.created_at);

    assert_eq!(elf.source, "Elf");

    assert_eq!(elf.targets, vec!["首选", "次选", "new"]);

    assert_eq!(elf.comment, "备注\nnew note");

    let unchanged = unchanged_document(&terms);

    let before =
        terminology::get_termbase(&coordinator, access.clone(), library.id.clone()).await?;

    for input in [unchanged, TermbaseAppend { terms: vec![] }] {
        let expected = u32::try_from(input.terms.len())?;

        assert_eq!(
            append_termbase(
                &coordinator,
                access.clone(),
                library.id.clone(),
                input,
                evidence_handle()
            )
            .await?,
            TermbaseAppendResult {
                unchanged: expected,
                ..TermbaseAppendResult::default()
            }
        );

        assert_eq!(
            terminology::get_termbase(&coordinator, access.clone(), library.id.clone()).await?,
            before
        );

        assert_eq!(
            terminology::list_term_infos(
                &coordinator,
                access.clone(),
                library.id.clone(),
                query("")
            )
            .await?,
            terms
        );
    }

    assert_eq!(before.termbase.name, library.name);

    assert_eq!(before.termbase.description, library.description);

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        terminology::list_term_infos(&reopened, access, library.id, query("")).await?,
        terms
    );

    reopened.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn failed_batch_rolls_back_prior_merges_and_inserts_and_rejects_duplicate_sources()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("test.sqlite")).await?);

    let access = TerminologyContext::Global;

    let library = base(&coordinator, access.clone(), "Library").await?;

    let original = term(&coordinator, access.clone(), &library, "Elf").await?;

    let before =
        terminology::get_termbase(&coordinator, access.clone(), library.id.clone()).await?;

    let mut terms = document().terms;

    terms.push(metadata("　"));

    assert_eq!(
        execute(
            &coordinator,
            AppendTermbase {
                access: access.clone(),
                termbase_id: library.id.clone(),
                terms
            },
            RecoveryScope {
                termbase_ids: vec![library.id.clone()],
                termbase_terms: true,
                ..RecoveryScope::default()
            },
            evidence_handle()
        )
        .await,
        Err(AppError::InvalidInput)
    );

    assert_eq!(
        append_termbase(
            &coordinator,
            access.clone(),
            library.id.clone(),
            TermbaseAppend {
                terms: vec![metadata("a"), metadata(" A　")]
            },
            evidence_handle()
        )
        .await,
        Err(AppError::DuplicateTerm)
    );

    assert_eq!(
        terminology::get_termbase(&coordinator, access.clone(), library.id.clone()).await?,
        before
    );

    assert_eq!(
        terminology::list_term_infos(&coordinator, access, library.id, query("")).await?,
        vec![original]
    );

    coordinator.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn append_enforces_ownership_and_only_touches_local_comic_time() -> Result<(), Box<dyn Error>>
{
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("test.sqlite")).await?);

    let chapter = comic::create_comic(
        &coordinator,
        NewComic {
            comic: ComicMetadata {
                title: "Comic".to_owned(),
                author: String::new(),
            },
            chapter_title: "Chapter".to_owned(),
        },
    )
    .await?;

    let access = TerminologyContext::Chapter {
        chapter_id: chapter.id.clone(),
    };

    let global = base(&coordinator, TerminologyContext::Global, "Global").await?;

    let local = base(&coordinator, access.clone(), "Local").await?;

    assert_eq!(
        append_termbase(
            &coordinator,
            access.clone(),
            global.id,
            document(),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );

    assert_eq!(
        append_termbase(
            &coordinator,
            TerminologyContext::Global,
            local.id.clone(),
            document(),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );

    let other = comic::create_comic(
        &coordinator,
        NewComic {
            comic: ComicMetadata {
                title: "Other".to_owned(),
                author: String::new(),
            },
            chapter_title: "Chapter".to_owned(),
        },
    )
    .await?;

    assert_eq!(
        append_termbase(
            &coordinator,
            TerminologyContext::Chapter {
                chapter_id: other.id
            },
            local.id.clone(),
            document(),
            evidence_handle()
        )
        .await,
        Err(AppError::NotFound)
    );

    sqlx::query!(
        "UPDATE comic SET updated_at = 1 WHERE id = ?",
        chapter.comic_id
    )
    .execute(coordinator.pool())
    .await?;

    let snapshot = crate::usecase::chapter::get_chapter_detail(&coordinator, &chapter.id).await?;

    let before = comic::get_comic(&coordinator, &chapter.comic_id).await?;

    assert_eq!(
        append_termbase(
            &coordinator,
            access,
            local.id,
            document(),
            evidence_handle()
        )
        .await?
        .added,
        2
    );

    assert!(
        comic::get_comic(&coordinator, &chapter.comic_id)
            .await?
            .updated_at
            > before.updated_at
    );

    let after = crate::usecase::chapter::get_chapter_detail(&coordinator, &chapter.id).await?;

    assert_eq!(after.chapter, snapshot.chapter);

    assert_eq!(after.pages, snapshot.pages);

    assert_eq!(after.work_position, snapshot.work_position);

    coordinator.pool().close().await;

    Ok(())
}
