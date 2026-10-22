use std::error::Error;

use tempfile::tempdir;

use crate::complex::terminology::import::parse;
use crate::data::comic::{ComicMetadata, MoveChapter, NewComic};
use crate::data::recovery::evidence_handle;
use crate::data::terminology::{
    TermMetadata, TermbaseImport, TermbaseMetadata, TerminologyContext,
};
use crate::implementation::coordinator::{SqliteCoordinator, database};
use crate::implementation::repository::terminology::test::{base, query};
use crate::result::AppError;
use crate::usecase::comic;
use crate::usecase::terminology;
use crate::usecase::terminology::export::export_termbase;
use crate::usecase::terminology::import::import_termbase;

#[tokio::test]
async fn complete_export_is_ordered_readonly_and_reimports_with_new_identities()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let coordinator = SqliteCoordinator::new(database::open(&directory.path().join("db")).await?);

    let access = TerminologyContext::Global;

    let document = TermbaseImport {
        metadata: TermbaseMetadata {
            name: "称谓".to_owned(),
            description: String::new(),
        },
        terms: (0..135)
            .rev()
            .map(|index| TermMetadata {
                source: format!("団長{index:03}"),
                targets: vec!["团长".to_owned(), "队长".to_owned()],
                comment: String::new(),
            })
            .collect(),
    };

    let base = import_termbase(&coordinator, access.clone(), document, evidence_handle()).await?;

    let before =
        terminology::list_term_infos(&coordinator, access.clone(), base.id.clone(), query(""))
            .await?;

    let exported = export_termbase(&coordinator, access.clone(), base.id.clone()).await?;

    assert_eq!(exported.terms.len(), 135);

    assert_eq!(exported.terms[0].source, "団長000");

    assert_eq!(exported.terms[134].source, "団長134");

    assert_eq!(exported.terms[0].targets, vec!["团长", "队长"]);

    assert!(exported.description.is_none());

    assert!(exported.terms[0].comment.is_none());

    assert_eq!(
        terminology::get_termbase(&coordinator, access.clone(), base.id.clone())
            .await?
            .termbase,
        base
    );

    assert_eq!(
        terminology::list_term_infos(&coordinator, access.clone(), base.id.clone(), query(""))
            .await?,
        before
    );

    let bytes = serde_json::to_vec(&exported)?;

    let mut imported = parse(&bytes)?;

    imported.metadata.name = "副本".to_owned();

    let copy = import_termbase(&coordinator, access.clone(), imported, evidence_handle()).await?;

    assert_ne!(copy.id, base.id);

    assert_eq!(
        export_termbase(&coordinator, access.clone(), copy.id.clone())
            .await?
            .terms,
        exported.terms
    );

    let copied =
        terminology::list_term_infos(&coordinator, access.clone(), copy.id, query("")).await?;

    assert!(
        copied
            .iter()
            .all(|term| before.iter().all(|original| term.id != original.id))
    );

    terminology::delete_term(
        &coordinator,
        access.clone(),
        before[0].clone(),
        evidence_handle(),
    )
    .await?;

    assert_eq!(exported.terms.len(), 135);

    assert_eq!(
        export_termbase(&coordinator, access, base.id)
            .await?
            .terms
            .len(),
        134
    );

    coordinator.pool().close().await;

    Ok(())
}

#[tokio::test]
async fn export_checks_current_read_scope_and_accepts_empty_libraries() -> Result<(), Box<dyn Error>>
{
    let directory = tempdir()?;

    let coordinator = SqliteCoordinator::new(database::open(&directory.path().join("db")).await?);

    let chapter = comic::create_comic(
        &coordinator,
        NewComic {
            comic: ComicMetadata {
                title: "漫画".to_owned(),
                author: String::new(),
            },
            chapter_title: "第一章".to_owned(),
        },
    )
    .await?;

    let access = TerminologyContext::Chapter {
        chapter_id: chapter.id.clone(),
    };

    let global = base(&coordinator, TerminologyContext::Global, "全局").await?;

    let local = base(&coordinator, access.clone(), "本作").await?;

    let target = comic::create_comic(
        &coordinator,
        NewComic {
            comic: ComicMetadata {
                title: "目标漫画".to_owned(),
                author: String::new(),
            },
            chapter_title: "第一章".to_owned(),
        },
    )
    .await?;

    let other_access = TerminologyContext::Chapter {
        chapter_id: target.id.clone(),
    };

    let target_base = base(&coordinator, other_access.clone(), "目标库").await?;

    assert_eq!(
        export_termbase(&coordinator, other_access, local.id.clone()).await,
        Err(AppError::NotFound)
    );

    assert!(
        export_termbase(&coordinator, access.clone(), global.id)
            .await?
            .terms
            .is_empty()
    );

    assert!(
        export_termbase(&coordinator, access.clone(), local.id.clone())
            .await?
            .terms
            .is_empty()
    );

    assert_eq!(
        export_termbase(&coordinator, TerminologyContext::Global, local.id.clone()).await,
        Err(AppError::NotFound)
    );

    comic::move_chapter(
        &coordinator,
        MoveChapter {
            chapter_id: chapter.id.clone(),
            source_comic_id: chapter.comic_id,
            target_comic_id: target.comic_id,
            source_baseline: vec![chapter.id],
            target_baseline: vec![target.id],
        },
    )
    .await?;

    assert_eq!(
        export_termbase(&coordinator, access.clone(), local.id).await,
        Err(AppError::NotFound)
    );

    assert!(
        export_termbase(&coordinator, access, target_base.id)
            .await?
            .terms
            .is_empty()
    );

    assert_eq!(
        export_termbase(
            &coordinator,
            TerminologyContext::Global,
            uuid::Uuid::new_v4().to_string()
        )
        .await,
        Err(AppError::NotFound)
    );

    coordinator.pool().close().await;

    Ok(())
}
