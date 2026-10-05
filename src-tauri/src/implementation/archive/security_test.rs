use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::implementation::archive::package;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn zip(entries: &[(&str, &[u8])]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));

    for (name, bytes) in entries {
        writer.start_file(
            *name,
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )?;

        writer.write_all(bytes)?;
    }

    Ok(writer.finish()?.into_inner())
}

#[test]
fn zip_rejects_path_escape_and_normalization_collisions() -> TestResult {
    for names in [
        vec!["../escape"],
        vec!["C:/escape"],
        vec!["\\\\host\\escape"],
        vec!["A.png", "a.png"],
        vec!["é.png", "e\u{301}.png"],
    ] {
        let entries = names
            .iter()
            .map(|name| (*name, b"x".as_slice()))
            .collect::<Vec<_>>();

        assert!(package::open(Cursor::new(zip(&entries)?)).is_err());
    }

    Ok(())
}

#[test]
fn invalid_prk_does_not_fall_back_to_valid_lp() -> TestResult {
    let bytes = zip(&[
        ("translation.prk.json", b"invalid"),
        ("translation.lp.txt", b"1,0\n-\ngroup\n-\nproducer\n"),
    ])?;

    let mut package = package::open(Cursor::new(bytes))?;

    assert!(
        package::read_document(&mut package, &crate::data::search::TextStage::Translation).is_err()
    );

    Ok(())
}

#[test]
fn corrupted_crc_is_rejected_when_consumed() -> TestResult {
    let mut bytes = zip(&[("image.png", b"UNIQUE_IMAGE_BYTES")])?;

    let offset = bytes
        .windows(18)
        .position(|slice| slice == b"UNIQUE_IMAGE_BYTES")
        .ok_or("missing fixture content")?;

    bytes[offset] ^= 1;

    let mut package = package::open(Cursor::new(bytes))?;

    assert!(package::copy_entry(&mut package, 0, &mut Vec::new(), 100).is_err());

    Ok(())
}

#[test]
fn extraction_budget_is_enforced_before_write() -> TestResult {
    let mut package = package::open(Cursor::new(zip(&[("image.png", b"12345")])?))?;

    let mut bytes = Vec::new();

    assert!(package::copy_entry(&mut package, 0, &mut bytes, 4).is_err());

    assert!(bytes.is_empty());

    Ok(())
}

#[test]
fn forged_directory_size_is_rejected_before_library_parse() -> TestResult {
    let mut bytes = zip(&[])?;

    let offset = bytes
        .windows(4)
        .rposition(|slice| slice == b"PK\x05\x06")
        .ok_or("missing fixture directory")?;

    bytes[offset + 12..offset + 16].copy_from_slice(&100_000_000_u32.to_le_bytes());

    assert!(package::open(Cursor::new(bytes)).is_err());

    Ok(())
}

#[test]
fn symlink_entries_are_rejected() -> TestResult {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));

    writer.add_symlink(
        "images/001.png",
        "../../private",
        SimpleFileOptions::default(),
    )?;

    assert!(package::open(Cursor::new(writer.finish()?.into_inner())).is_err());

    Ok(())
}
