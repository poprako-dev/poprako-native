use std::io::Cursor;

use crate::complex::archive::{json, label_plus, native};
use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchiveImage, ArchivePage, ArchiveUnit};
use crate::implementation::archive::package;
use crate::result::AppResult;
use crate::value::image::ImageFormat;

fn fixture() -> ArchiveDocument {
    ArchiveDocument {
        comic: ArchiveComic {
            title: "书".to_owned(),
            subtitle: String::new(),
            author: String::new(),
        },
        pages: vec![ArchivePage {
            index: 0,
            source_image_path: Some("images/001.png".to_owned()),
            image: Some(ArchiveImage {
                path: "images/001.png".to_owned(),
                original_name: "原图.png".to_owned(),
                format: ImageFormat::Png,
                width: 2,
                height: 3,
            }),
            units: vec![ArchiveUnit {
                index: 0,
                x_coord: 0.25,
                y_coord: 0.5,
                is_bubble: true,
                is_flagged: true,
                translated_text: "  a\n\nb ".to_owned(),
                proofread_text: "校对".to_owned(),
                is_proofread: false,
            }],
        }],
        warnings: Vec::new(),
    }
}

#[test]
fn native_preserves_both_stages_and_independent_confirmation() -> AppResult<()> {
    let document = fixture();

    let parsed = json::parse_prk(&native::encode(&document)?)?;

    assert_eq!(document, parsed);

    Ok(())
}

#[test]
fn rejects_duplicate_keys_and_unknown_native_version() {
    assert!(json::parse_prk(br#"{"format":"poprako-native","format":"other"}"#).is_err());

    assert!(
        json::parse_prk(
            br#"{"format":"poprako-native","format_version":2,"comic_title":"legacy"}"#
        )
        .is_err()
    );
}

#[test]
fn lp_uses_final_text_and_rejects_structure_in_body() -> AppResult<()> {
    let mut document = fixture();

    let lp = label_plus::encode(&document)?;

    let parsed = label_plus::parse(&lp)?;

    assert_eq!(parsed.pages[0].units[0].translated_text, "校对");

    assert!(!parsed.pages[0].units[0].is_proofread);

    document.pages[0].units[0].proofread_text = ">>>>>>>>[evil]<<<<<<<<".to_owned();

    assert!(label_plus::encode(&document).is_err());

    Ok(())
}

#[test]
fn lp_accepts_bom_crlf_and_reindexes_gaps() -> AppResult<()> {
    let document = label_plus::parse("\u{feff}1,0\r\n-\r\n框内\r\n-\r\nProducer\r\n>>>>>>>>[1.png]<<<<<<<< \t\r\n----------------[3]----------------[0,1,2]\r\na\r\n\r\nb".as_bytes())?;

    assert_eq!(document.pages[0].units[0].index, 0);

    assert_eq!(document.pages[0].units[0].translated_text, "a\nb");

    Ok(())
}

#[test]
fn zip_preserves_raw_image_bytes_and_prefers_prk() -> AppResult<()> {
    let bytes = vec![0, 1, 2, 255, 0, 3];

    let writer = package::write_package(Cursor::new(Vec::new()), &fixture(), true, |_| {
        Ok(Cursor::new(bytes.clone()))
    })?;

    let mut package = package::open(Cursor::new(writer.into_inner()))?;

    let parsed =
        package::read_document(&mut package, &crate::data::search::TextStage::Translation)?;

    let mut image = Vec::new();

    package::copy_entry(&mut package, 2, &mut image, 100)?;

    assert_eq!(image, bytes);

    assert!(parsed.pages[0].units[0].is_flagged);

    Ok(())
}

#[test]
fn lp_proofreading_import_does_not_infer_confirmation() -> AppResult<()> {
    let parsed = label_plus::parse_for_stage(
        &label_plus::encode(&fixture())?,
        &crate::data::search::TextStage::Proofreading,
    )?;

    assert!(parsed.pages[0].units[0].translated_text.is_empty());

    assert_eq!(parsed.pages[0].units[0].proofread_text, "校对");

    assert!(!parsed.pages[0].units[0].is_proofread);

    Ok(())
}

#[test]
fn legacy_snake_and_camel_are_strictly_separate() -> AppResult<()> {
    let snake = br#"{"comic_id":"c","comic_title":"T","chapter_id":"h","chapter_index":0,"pages":[{"page_id":"p","page_index":0,"units":[{"unit_id":"u","unit_index":4,"page_id":"p","page_index":0,"x_coord":0.5,"y_coord":0.5,"is_bubble":true,"is_proofread":false,"translated_text":null,"proofread_text":"P"}]}]}"#;

    let camel = br#"{"comicId":"c","comicTitle":"T","chapterId":"h","chapterIndex":0,"pages":[{"pageId":"p","pageIndex":0,"units":[{"unitId":"u","unitIndex":4,"pageId":"p","pageIndex":0,"xCoord":0.5,"yCoord":0.5,"isBubble":true,"isProofread":false,"translatedText":null,"proofreadText":"P"}]}]}"#;

    let first = json::parse_prk(snake)?;

    let second = json::parse_prk(camel)?;

    assert_eq!(first, second);

    assert_eq!(first.pages[0].units[0].index, 0);

    assert!(first.pages[0].units[0].translated_text.is_empty());

    assert!(!first.pages[0].units[0].is_flagged);

    let mixed =
        String::from_utf8_lossy(camel).replace("\"isProofread\":false", "\"is_proofread\":false");

    assert!(json::parse_prk(mixed.as_bytes()).is_err());

    Ok(())
}

#[test]
fn prk_rejects_nested_duplicate_keys_and_trailing_data() -> AppResult<()> {
    let encoded = native::encode(&fixture())?;

    let text = String::from_utf8_lossy(&encoded);

    for key in ["title", "width", "x_coord"] {
        let quoted = format!("\"{key}\":");

        let duplicate = text.replacen(&quoted, &format!("{quoted}null,{quoted}"), 1);

        assert_ne!(duplicate, text);

        assert!(json::parse_prk(duplicate.as_bytes()).is_err());
    }

    let mut trailing = encoded.clone();

    trailing.extend_from_slice(b" {}");

    assert!(json::parse_prk(&trailing).is_err());

    let mut whitespace = encoded;

    whitespace.extend_from_slice(b" \n\t");

    assert_eq!(json::parse_prk(&whitespace)?, fixture());

    Ok(())
}
