use std::error::Error;

use tempfile::tempdir;

use crate::data::recovery::evidence_handle;
use crate::data::terminology::{TermMetadata, TermbaseMetadata, TerminologyContext};
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::implementation::repository::terminology::test::{base, metadata, query, term};
use crate::result::AppError;
use crate::usecase::terminology;

async fn assert_invalid_terms(
    coordinator: &SqliteCoordinator,
    access: &TerminologyContext,
    library: &crate::model::terminology::Termbase,
) {
    for invalid in [
        TermMetadata {
            targets: vec![],
            ..metadata("another")
        },
        TermMetadata {
            targets: vec!["a".to_owned(), " A ".to_owned()],
            ..metadata("another")
        },
        metadata("　"),
    ] {
        assert_eq!(
            terminology::create_term(
                coordinator,
                access.clone(),
                library.id.clone(),
                invalid,
                evidence_handle()
            )
            .await,
            Err(AppError::InvalidInput)
        );
    }
}

#[tokio::test]
async fn pagination_is_stable_and_library_edits_check_the_baseline() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("test.sqlite")).await?);

    let access = TerminologyContext::Global;

    let original = base(&coordinator, access.clone(), "词典").await?;

    let mut entries = Vec::new();

    for index in 0..31 {
        entries.push(
            term(
                &coordinator,
                access.clone(),
                &original,
                &format!("word{index}"),
            )
            .await?,
        );
    }

    entries.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut listed = terminology::list_term_infos(
        &coordinator,
        access.clone(),
        original.id.clone(),
        query("WORD"),
    )
    .await?;

    assert_eq!(listed.len(), 30);

    let mut next = query("WORD");

    next.offset = 30;

    listed.extend(
        terminology::list_term_infos(&coordinator, access.clone(), original.id.clone(), next)
            .await?,
    );

    assert_eq!(listed, entries);

    let baseline = terminology::get_termbase(&coordinator, access.clone(), original.id)
        .await?
        .termbase;

    let renamed = terminology::update_termbase(
        &coordinator,
        access.clone(),
        baseline.clone(),
        TermbaseMetadata {
            name: "　名称変更　".to_owned(),
            description: " new ".to_owned(),
        },
        evidence_handle(),
    )
    .await?;

    assert_eq!(renamed.name, "名称変更");

    assert_eq!(renamed.description, "new");

    assert_eq!(
        terminology::delete_termbase(&coordinator, access.clone(), baseline, evidence_handle(),)
            .await,
        Err(AppError::Conflict)
    );

    assert_eq!(
        terminology::update_termbase(
            &coordinator,
            access,
            renamed.clone(),
            TermbaseMetadata {
                name: renamed.name.clone(),
                description: renamed.description.clone()
            },
            evidence_handle(),
        )
        .await?,
        renamed
    );

    coordinator.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn normalization_order_literal_queries_stale_edits_and_commit_recovery()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("test.sqlite")).await?);

    let access = TerminologyContext::Global;

    let library = base(&coordinator, access.clone(), "  FANTASY　").await?;

    assert_eq!(library.name, "FANTASY");

    assert_eq!(
        terminology::create_termbase(
            &coordinator,
            access.clone(),
            TermbaseMetadata {
                name: "fantasy".to_owned(),
                description: String::new()
            },
            evidence_handle()
        )
        .await,
        Err(AppError::DuplicateTermbase)
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let entry = term(&coordinator, access.clone(), &library, "　Élf%_　").await?;

    assert_eq!(entry.source, "Élf%_");

    assert_eq!(
        terminology::create_term(
            &coordinator,
            access.clone(),
            library.id.clone(),
            metadata("élf%_"),
            evidence_handle()
        )
        .await,
        Err(AppError::DuplicateTerm)
    );

    assert_invalid_terms(&coordinator, &access, &library).await;

    term(&coordinator, access.clone(), &library, "regular").await?;

    assert_eq!(
        terminology::list_term_infos(
            &coordinator,
            access.clone(),
            library.id.clone(),
            query("%_")
        )
        .await?,
        vec![entry.clone()]
    );

    let changed = TermMetadata {
        targets: vec!["次选".to_owned(), "首选".to_owned()],
        ..metadata("Élf%_")
    };

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        terminology::update_term(
            &coordinator,
            access.clone(),
            entry.clone(),
            changed.clone(),
            evidence_handle()
        )
        .await,
        Err(AppError::Storage)
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let saved = terminology::update_term(
        &coordinator,
        access.clone(),
        entry.clone(),
        changed,
        evidence_handle(),
    )
    .await?;

    assert_eq!(
        terminology::preferred_translation(&coordinator, access.clone(), saved.id.clone()).await?,
        "次选"
    );

    assert_eq!(
        terminology::delete_term(&coordinator, access.clone(), entry, evidence_handle()).await,
        Err(AppError::Conflict)
    );

    let before =
        terminology::get_termbase(&coordinator, access.clone(), library.id.clone()).await?;

    assert_eq!(before.term_count, 2);

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    terminology::delete_termbase(
        &coordinator,
        access.clone(),
        before.termbase,
        evidence_handle(),
    )
    .await?;

    assert!(
        terminology::list_termbase_infos(&coordinator, access, query(""))
            .await?
            .is_empty()
    );

    coordinator.pool().close().await;

    Ok(())
}
