use std::error::Error;

use crate::complex::terminology::append::{merge, parse};
use crate::complex::terminology::import::MAX_FILE_BYTES;
use crate::data::terminology::TermMetadata;
use crate::result::AppError;

fn term(source: &str, targets: &[&str], comment: &str) -> TermMetadata {
    TermMetadata {
        source: source.to_owned(),
        targets: targets.iter().map(|value| (*value).to_owned()).collect(),
        comment: comment.to_owned(),
    }
}

#[test]
fn append_parses_trimmed_terms_without_metadata() -> Result<(), Box<dyn Error>> {
    let input = parse("\u{feff}{\"terms\":[{\"source\":\"　Elf　\",\"targets\":[\" 精灵　\",\" 妖精 \"],\"comment\":\" 注释　\"}]}".as_bytes())?;

    assert_eq!(input.terms, vec![term("Elf", &["精灵", "妖精"], "注释")]);

    assert!(parse(br#"{"terms":[]}"#)?.terms.is_empty());

    for comment in ["", ",\"comment\":null"] {
        let input = format!(r#"{{"terms":[{{"source":"x","targets":["y"]{comment}}}]}}"#);

        assert_eq!(parse(input.as_bytes())?.terms[0].comment, "");
    }

    Ok(())
}

#[test]
fn strict_append_rejects_metadata_duplicate_keys_and_invalid_batches() -> Result<(), Box<dyn Error>>
{
    for content in [
        "{broken",
        "null",
        "[]",
        "{}",
        r#"{"terms":[],"name":"library"}"#,
        r#"{"terms":[],"description":null}"#,
        r#"{"terms":[],"metadata":{}}"#,
        r#"{"terms":[],"terms":[]}"#,
        r#"{"terms":[{"source":"x","targets":["y"],"source":"z"}]}"#,
        r#"{"terms":[{"source":"x","targets":["y"],"extra":0}]}"#,
        r#"{"terms":[{"source":" ","targets":["y"]}]}"#,
        r#"{"terms":[{"source":"x","targets":[]}]}"#,
        r#"{"terms":[{"source":"x","targets":["　"]}]}"#,
        r#"{"terms":[{"source":"x","targets":["y"," Y "]}]}"#,
        r#"{"terms":[{"source":"Élf","targets":["y"]},{"source":" éLF ","targets":["z"]}]}"#,
    ] {
        assert_eq!(parse(content.as_bytes()), Err(AppError::TermbaseImport));
    }

    assert_eq!(parse(&[0xff]), Err(AppError::TermbaseImport));

    assert_eq!(
        parse(&vec![b' '; usize::try_from(MAX_FILE_BYTES + 1)?]),
        Err(AppError::TermbaseImport)
    );

    Ok(())
}

#[test]
fn merge_trims_both_sides_and_preserves_preferred_translation() -> Result<(), Box<dyn Error>> {
    let merged = merge(
        term(" Elf　", &[" Preferred ", " Second "], " Old　"),
        term(
            " eLF　",
            &[" preferred　", " Third　", " second "],
            " New　",
        ),
    )?;

    assert_eq!(
        merged,
        term("Elf", &["Preferred", "Second", "Third"], "Old\nNew")
    );

    assert!(merge(term("x", &["y"], ""), term("z", &["y"], "")).is_err());

    Ok(())
}

#[test]
fn comments_compare_whole_trimmed_blocks_without_line_deduplication() -> Result<(), Box<dyn Error>>
{
    for (before, incoming, expected) in [
        (" old　", "　", "old"),
        ("　", " new ", "new"),
        (" old　", " old ", "old"),
        ("　", " ", ""),
        (" A\nB　", " A\nB ", "A\nB"),
        (" A\nB ", " B\nC ", "A\nB\nB\nC"),
        (" A\nB ", " B ", "A\nB\nB"),
    ] {
        assert_eq!(
            merge(term("x", &["y"], before), term("X", &["Y"], incoming))?.comment,
            expected
        );
    }

    Ok(())
}
