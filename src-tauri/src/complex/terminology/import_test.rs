use std::error::Error;

use crate::complex::terminology::import::{MAX_FILE_BYTES, parse};
use crate::result::AppError;

#[test]
fn portable_json_preserves_translation_order_and_normalizes_optional_text()
-> Result<(), Box<dyn Error>> {
    let parsed = parse("\u{feff}{\"name\":\"　称谓　\",\"description\":null,\"terms\":[{\"source\":\" Elf \",\"targets\":[\" 精灵 \",\"妖精\"],\"comment\":null}]}".as_bytes())?;

    assert_eq!(parsed.metadata.name, "称谓");

    assert!(parsed.metadata.description.is_empty());

    assert_eq!(parsed.terms[0].source, "Elf");

    assert_eq!(parsed.terms[0].targets, vec!["精灵", "妖精"]);

    assert!(parsed.terms[0].comment.is_empty());

    assert!(parse(br#"{"name":"empty","terms":[]}"#)?.terms.is_empty());

    Ok(())
}

#[test]
fn malformed_duplicate_and_oversized_imports_are_rejected() -> Result<(), Box<dyn Error>> {
    for content in [
        "{broken",
        "null",
        "{}",
        r#"{"name":"a","name":"b","terms":[]}"#,
        r#"{"name":"a","terms":[],"comic_id":"forged"}"#,
        r#"{"name":"a","terms":[{"source":"x","targets":[] }]}"#,
        r#"{"name":"a","terms":[{"source":"x","targets":["y"," Y "]}]}"#,
        r#"{"name":"a","terms":[{"source":"Élf","targets":["y"]},{"source":" éLF ","targets":["z"]}]}"#,
    ] {
        assert_eq!(parse(content.as_bytes()), Err(AppError::TermbaseImport));
    }

    assert_eq!(
        parse(&vec![b' '; usize::try_from(MAX_FILE_BYTES + 1)?]),
        Err(AppError::TermbaseImport)
    );

    Ok(())
}
