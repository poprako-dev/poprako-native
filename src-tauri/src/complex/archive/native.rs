use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::complex::archive::path::logical_path;
use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchiveImage, ArchivePage, ArchiveUnit};
use crate::result::{AppError, AppResult};
use crate::value::validation::{normalize_body, position, validate_coordinate};

#[derive(Serialize, Deserialize)]
struct NativePage {
    index: u32,
    image: ArchiveImage,
    units: Vec<ArchiveUnit>,
}

#[derive(Serialize, Deserialize)]
struct NativeDocument {
    format: String,
    format_version: u32,
    coordinate_space: String,
    comic: ArchiveComic,
    pages: Vec<NativePage>,
}

/// # Errors
/// Rejects invalid coordinates and duplicate or discontinuous order.
pub fn normalize_units(units: &mut [ArchiveUnit], contiguous: bool) -> AppResult<()> {
    units.sort_by_key(|unit| unit.index);

    let mut seen = HashSet::new();

    for (index, unit) in units.iter_mut().enumerate() {
        if !seen.insert(unit.index) || (contiguous && unit.index != position(index)?) {
            return Err(AppError::InvalidInput);
        }

        validate_coordinate(unit.x_coord)?;

        validate_coordinate(unit.y_coord)?;

        unit.index = position(index)?;

        unit.translated_text = normalize_body(&unit.translated_text);

        unit.proofread_text = normalize_body(&unit.proofread_text);
    }

    Ok(())
}

fn unknown_fields(value: &Value, allowed: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.keys().any(|key| !allowed.contains(&key.as_str())))
}

fn has_unknown(value: &Value) -> bool {
    if unknown_fields(
        value,
        &[
            "format",
            "format_version",
            "coordinate_space",
            "comic",
            "pages",
        ],
    ) || unknown_fields(&value["comic"], &["title", "subtitle", "author"])
    {
        return true;
    }

    value["pages"].as_array().is_some_and(|pages| {
        pages.iter().any(|page| {
            unknown_fields(page, &["index", "image", "units"])
                || unknown_fields(
                    &page["image"],
                    &["path", "original_name", "format", "width", "height"],
                )
                || page["units"].as_array().is_some_and(|units| {
                    units.iter().any(|unit| {
                        unknown_fields(
                            unit,
                            &[
                                "index",
                                "x_coord",
                                "y_coord",
                                "is_bubble",
                                "is_flagged",
                                "translated_text",
                                "proofread_text",
                                "is_proofread",
                            ],
                        )
                    })
                })
        })
    })
}

/// # Errors
/// Rejects unsupported native versions and inconsistent image declarations.
pub fn parse(value: Value) -> AppResult<ArchiveDocument> {
    let extra = has_unknown(&value);

    let mut native: NativeDocument =
        serde_json::from_value(value).map_err(|_| AppError::InvalidInput)?;

    if native.format != "poprako-native"
        || native.format_version != 1
        || native.coordinate_space != "display-oriented"
    {
        return Err(AppError::InvalidInput);
    }

    native.comic = ArchiveComic {
        title: native.comic.title.trim().to_owned(),
        subtitle: native.comic.subtitle.trim().to_owned(),
        author: native.comic.author.trim().to_owned(),
    };

    if native.comic.title.is_empty() {
        return Err(AppError::InvalidInput);
    }

    native.pages.sort_by_key(|page| page.index);

    let mut pages = Vec::new();

    let mut paths = HashSet::new();

    for (index, mut page) in native.pages.into_iter().enumerate() {
        if page.index != position(index)?
            || page.image.width == 0
            || page.image.height == 0
            || page.image.width > 16384
            || page.image.height > 16384
            || u64::from(page.image.width) * u64::from(page.image.height) > 32_000_000
            || !paths.insert(logical_path(&page.image.path)?)
        {
            return Err(AppError::InvalidInput);
        }

        normalize_units(&mut page.units, true)?;

        pages.push(ArchivePage {
            index: page.index,
            source_image_path: Some(page.image.path.clone()),
            image: Some(page.image),
            units: page.units,
        });
    }

    let warnings = match extra {
        true => vec!["已忽略来源中的未知附加字段".to_owned()],
        false => Vec::new(),
    };

    Ok(ArchiveDocument {
        comic: native.comic,
        pages,
        warnings,
    })
}

/// # Errors
/// Requires a complete image declaration for every exported page.
pub fn encode(document: &ArchiveDocument) -> AppResult<Vec<u8>> {
    let pages = document
        .pages
        .iter()
        .map(|page| {
            Ok(NativePage {
                index: page.index,
                image: page.image.clone().ok_or(AppError::InvalidInput)?,
                units: page.units.clone(),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;

    let native = NativeDocument {
        format: "poprako-native".to_owned(),
        format_version: 1,
        coordinate_space: "display-oriented".to_owned(),
        comic: document.comic.clone(),
        pages,
    };

    let value = serde_json::to_value(&native).map_err(|_| AppError::InvalidInput)?;

    parse(value)?;

    serde_json::to_vec_pretty(&native).map_err(|_| AppError::InvalidInput)
}
