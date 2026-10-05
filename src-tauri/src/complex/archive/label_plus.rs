use std::fmt::Write;

use crate::complex::archive::json::METADATA_LIMIT;
use crate::complex::archive::native::normalize_units;
use crate::complex::archive::path::logical_path;
use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchivePage, ArchiveUnit};
use crate::result::{AppError, AppResult};
use crate::value::validation::{normalize_body, position, validate_coordinate};

fn structure(line: &str) -> &str {
    line.trim_end_matches([' ', '\t'])
}

fn header(line: &str) -> AppResult<ArchiveUnit> {
    let (ordinal, coordinates) = structure(line)
        .strip_prefix("----------------[")
        .and_then(|line| line.split_once("]----------------["))
        .ok_or(AppError::InvalidInput)?;

    let index = ordinal
        .parse::<u32>()
        .ok()
        .and_then(|index| index.checked_sub(1))
        .ok_or(AppError::InvalidInput)?;

    let parts = coordinates
        .strip_suffix(']')
        .ok_or(AppError::InvalidInput)?
        .split(',')
        .collect::<Vec<_>>();

    let [x, y, group] = parts.as_slice() else {
        return Err(AppError::InvalidInput);
    };

    let x_coord = x.parse::<f64>().map_err(|_| AppError::InvalidInput)?;

    let y_coord = y.parse::<f64>().map_err(|_| AppError::InvalidInput)?;

    validate_coordinate(x_coord)?;

    validate_coordinate(y_coord)?;

    let is_bubble = match *group {
        "1" => true,
        "2" => false,
        _ => return Err(AppError::InvalidInput),
    };

    Ok(ArchiveUnit {
        index,
        x_coord,
        y_coord,
        is_bubble,
        is_flagged: false,
        translated_text: String::new(),
        proofread_text: String::new(),
        is_proofread: false,
    })
}

fn finish(page: &mut ArchivePage) -> AppResult<()> {
    for unit in &mut page.units {
        unit.translated_text = normalize_body(&unit.translated_text);
    }

    normalize_units(&mut page.units, false)
}

/// # Errors
/// Rejects invalid UTF-8, malformed structure and ambiguous page names.
pub fn parse(bytes: &[u8]) -> AppResult<ArchiveDocument> {
    if bytes.len() > METADATA_LIMIT || bytes.contains(&0) {
        return Err(AppError::InvalidInput);
    }

    let content = std::str::from_utf8(bytes).map_err(|_| AppError::InvalidInput)?;

    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    let mut lines = content
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line));

    if !lines
        .next()
        .and_then(|line| line.chars().next())
        .is_some_and(|character| character.is_ascii_digit())
        || lines.next().map(structure) != Some("-")
        || !lines.by_ref().any(|line| structure(line) == "-")
        || lines.next().is_none()
    {
        return Err(AppError::InvalidInput);
    }

    let mut pages: Vec<ArchivePage> = Vec::new();

    for line in lines {
        if structure(line).starts_with(">>>>>>>>") {
            let filename = structure(line)
                .strip_prefix(">>>>>>>>[")
                .and_then(|line| line.strip_suffix("]<<<<<<<<"))
                .ok_or(AppError::InvalidInput)?;

            logical_path(filename)?;

            pages.push(ArchivePage {
                index: position(pages.len())?,
                image: None,
                source_image_path: Some(filename.to_owned()),
                units: Vec::new(),
            });

            continue;
        }

        if structure(line).starts_with("----------------[") {
            pages
                .last_mut()
                .ok_or(AppError::InvalidInput)?
                .units
                .push(header(line)?);

            continue;
        }

        if line.is_empty() {
            continue;
        }

        if let Some(unit) = pages.last_mut().and_then(|page| page.units.last_mut()) {
            if !unit.translated_text.is_empty() {
                unit.translated_text.push('\n');
            }

            unit.translated_text.push_str(line);
        }
    }

    for page in &mut pages {
        finish(page)?;
    }

    Ok(ArchiveDocument {
        comic: ArchiveComic {
            title: String::new(),
            subtitle: String::new(),
            author: String::new(),
        },
        pages,
        warnings: vec![
            "LP 仅含单阶段正文；空行、坐标精度、关注及校对确认无法完整恢复，请核对图片方向"
                .to_owned(),
        ],
    })
}

/// # Errors
/// Rejects body lines that a compatible parser would mistake for structure.
pub fn encode(document: &ArchiveDocument) -> AppResult<Vec<u8>> {
    let mut output = "1,0\n-\n框内\n框外\n-\nExported by PopRaKo Native\n".to_owned();

    for page in &document.pages {
        let image = page.image.as_ref().ok_or(AppError::InvalidInput)?;

        let name = image
            .path
            .rsplit('/')
            .next()
            .ok_or(AppError::InvalidInput)?;

        logical_path(name)?;

        writeln!(output, "\n\n>>>>>>>>[{name}]<<<<<<<<").map_err(|_| AppError::Storage)?;

        for (index, unit) in page.units.iter().enumerate() {
            let text = match unit.proofread_text.is_empty() {
                true => &unit.translated_text,
                false => &unit.proofread_text,
            };

            if text.contains('\0')
                || text.lines().any(|line| {
                    line.starts_with(">>>>>>>>") || line.starts_with("----------------[")
                })
            {
                return Err(AppError::InvalidInput);
            }

            validate_coordinate(unit.x_coord)?;

            validate_coordinate(unit.y_coord)?;

            let group = match unit.is_bubble {
                true => 1,
                false => 2,
            };

            writeln!(
                output,
                "----------------[{}]----------------[{:.4},{:.4},{group}]\n{text}",
                index + 1,
                unit.x_coord,
                unit.y_coord
            )
            .map_err(|_| AppError::Storage)?;
        }
    }

    Ok(output.into_bytes())
}

/// # Errors
/// Parses LP into the explicitly selected stage without inferring confirmation.
pub fn parse_for_stage(
    bytes: &[u8],
    stage: &crate::data::search::TextStage,
) -> AppResult<ArchiveDocument> {
    let mut document = parse(bytes)?;

    if *stage == crate::data::search::TextStage::Proofreading {
        for page in &mut document.pages {
            for unit in &mut page.units {
                unit.proofread_text = std::mem::take(&mut unit.translated_text);
            }
        }
    }

    Ok(document)
}
