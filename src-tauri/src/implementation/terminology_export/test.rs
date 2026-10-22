use std::error::Error;
use std::fs;

use tempfile::tempdir;

use crate::complex::terminology::import::{MAX_FILE_BYTES, parse};
use crate::data::terminology::portable::{PortableTerm, PortableTermbase};
use crate::implementation::terminology_export::{filename, write};
use crate::result::AppError;

fn document() -> PortableTermbase {
    PortableTermbase {
        name: "称谓".to_owned(),
        description: None,
        terms: vec![PortableTerm {
            source: "団長".to_owned(),
            targets: vec!["团长".to_owned(), "队长".to_owned()],
            comment: None,
        }],
    }
}

#[test]
fn published_json_round_trips_and_existing_files_require_authorization()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let target = directory.path().join("称谓.json");

    fs::write(&target, "original")?;

    assert_eq!(write(&target, &document(), false), Err(AppError::Storage));

    assert_eq!(fs::read(&target)?, b"original");

    write(&target, &document(), true)?;

    let content = fs::read(&target)?;

    assert_eq!(
        serde_json::from_slice::<PortableTermbase>(&content)?,
        document()
    );

    let imported = parse(&content)?;

    assert_eq!(imported.metadata.name, "称谓");

    assert!(imported.metadata.description.is_empty());

    assert_eq!(imported.terms[0].targets, vec!["团长", "队长"]);

    assert!(imported.terms[0].comment.is_empty());

    assert_eq!(fs::read_dir(directory.path())?.count(), 1);

    Ok(())
}

#[test]
fn export_exceeds_the_import_limit_and_failed_publication_leaves_no_temporary_files()
-> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;

    let target = directory.path().join("large.json");

    let mut large = document();

    large.terms[0].comment = Some("x".repeat(usize::try_from(MAX_FILE_BYTES)?));

    write(&target, &large, false)?;

    let content = fs::read(&target)?;

    assert!(u64::try_from(content.len())? > MAX_FILE_BYTES);

    assert_eq!(parse(&content), Err(AppError::TermbaseImport));

    assert_eq!(serde_json::from_slice::<PortableTermbase>(&content)?, large);

    let blocked = directory.path().join("directory.json");

    fs::create_dir(&blocked)?;

    assert_eq!(write(&blocked, &document(), true), Err(AppError::Storage));

    assert!(blocked.is_dir());

    assert_eq!(fs::read_dir(directory.path())?.count(), 2);

    Ok(())
}

#[test]
fn default_names_are_portable_and_do_not_change_document_names() {
    assert_eq!(filename("称谓"), "称谓.json");

    assert_eq!(filename("../角色\\名称:?"), "_角色_名称__.json");

    assert_eq!(filename(" . "), "termbase.json");

    assert_eq!(filename("CON"), "_CON.json");

    assert_eq!(filename("LPT1.txt"), "_LPT1.txt.json");

    assert!(filename(&"术".repeat(100)).len() < 255);
}
