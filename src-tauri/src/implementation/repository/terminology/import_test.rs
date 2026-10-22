use std::error::Error;

use tempfile::tempdir;
use uuid::Uuid;

use crate::complex::terminology::import::parse;
use crate::data::recovery::{RecoveryScope, evidence_handle};
use crate::data::terminology::{
    TermMetadata, TermbaseImport, TermbaseMetadata, TerminologyContext,
};
use crate::implementation::coordinator::{CommitFault, SqliteCoordinator, database};
use crate::implementation::repository::terminology::test::query;
use crate::model::terminology::TermbaseScope;
use crate::part::repository::terminology::{CreateTerm, CreateTermbase, ImportTermbase};
use crate::result::AppError;
use crate::usecase::recovery::execute;
use crate::usecase::terminology;
use crate::usecase::terminology::import::import_termbase;

fn document() -> Result<TermbaseImport, Box<dyn Error>> {
    Ok(parse(br#"{"name":"Library","description":null,"terms":[{"source":"Elf","targets":["preferred","alternate"]},{"source":"Dwarf","targets":["dwarf"]}]}"#)?)
}

#[tokio::test]
async fn import_commit_recovery_conflicts_and_reopening_preserve_whole_libraries()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let path = directory.path().join("test.sqlite");

    let coordinator = SqliteCoordinator::new(database::open(&path).await?);

    coordinator.inject_commit_fault(CommitFault::LostRolledBack);

    assert_eq!(
        import_termbase(
            &coordinator,
            TerminologyContext::Global,
            document()?,
            evidence_handle()
        )
        .await,
        Err(AppError::Storage)
    );

    assert!(
        terminology::list_termbase_infos(&coordinator, TerminologyContext::Global, query(""))
            .await?
            .is_empty()
    );

    coordinator.inject_commit_fault(CommitFault::LostCommitted);

    let base = import_termbase(
        &coordinator,
        TerminologyContext::Global,
        document()?,
        evidence_handle(),
    )
    .await?;

    assert_eq!(
        import_termbase(
            &coordinator,
            TerminologyContext::Global,
            document()?,
            evidence_handle()
        )
        .await,
        Err(AppError::DuplicateTermbase)
    );

    let terms = terminology::list_term_infos(
        &coordinator,
        TerminologyContext::Global,
        base.id.clone(),
        query(""),
    )
    .await?;

    assert_eq!(terms.len(), 2);

    let elf = terms
        .iter()
        .find(|term| term.source == "Elf")
        .ok_or("missing Elf")?;

    assert_eq!(elf.targets, vec!["preferred", "alternate"]);

    coordinator.pool().close().await;

    let reopened = SqliteCoordinator::new(database::open(&path).await?);

    assert_eq!(
        terminology::list_term_infos(&reopened, TerminologyContext::Global, base.id, query(""))
            .await?,
        terms
    );

    reopened.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn failure_after_an_insert_rolls_back_the_library_and_all_terms() -> Result<(), Box<dyn Error>>
{
    let directory = tempdir()?;

    let coordinator =
        SqliteCoordinator::new(database::open(&directory.path().join("test.sqlite")).await?);

    let id = Uuid::new_v4().to_string();

    let terms = ["valid", ""]
        .into_iter()
        .map(|source| CreateTerm {
            access: TerminologyContext::Global,
            termbase_id: id.clone(),
            id: Uuid::new_v4().to_string(),
            metadata: TermMetadata {
                source: source.to_owned(),
                targets: vec!["translation".to_owned()],
                comment: String::new(),
            },
        })
        .collect();

    let operation = ImportTermbase {
        base: CreateTermbase {
            access: TerminologyContext::Global,
            scope: TermbaseScope::Global,
            id: id.clone(),
            metadata: TermbaseMetadata {
                name: "Atomic".to_owned(),
                description: String::new(),
            },
        },
        terms,
    };

    let scope = RecoveryScope {
        termbase_ids: vec![id.clone()],
        termbase_terms: true,
        ..RecoveryScope::default()
    };

    assert_eq!(
        execute(&coordinator, operation, scope, evidence_handle()).await,
        Err(AppError::InvalidInput)
    );

    assert_eq!(
        terminology::get_termbase(&coordinator, TerminologyContext::Global, id).await,
        Err(AppError::NotFound)
    );

    assert!(
        terminology::list_termbase_infos(&coordinator, TerminologyContext::Global, query(""))
            .await?
            .is_empty()
    );

    coordinator.pool().close().await;

    Ok(())
}
