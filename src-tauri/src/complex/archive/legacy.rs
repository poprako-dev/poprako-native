use serde_json::Value;

use crate::complex::archive::native::normalize_units;
use crate::complex::archive::path::logical_path;
use crate::data::archive::{ArchiveComic, ArchiveDocument, ArchivePage, ArchiveUnit};
use crate::result::{AppError, AppResult};
use crate::value::validation::position;

fn key<'a>(snake: &'a str, camel: &'a str, web: bool) -> &'a str {
    match web {
        true => camel,
        false => snake,
    }
}

fn field<'a>(value: &'a Value, snake: &str, camel: &str, web: bool) -> AppResult<&'a Value> {
    if value.get(key(camel, snake, web)).is_some() {
        return Err(AppError::InvalidInput);
    }

    value
        .get(key(snake, camel, web))
        .ok_or(AppError::InvalidInput)
}

fn text(value: &Value) -> AppResult<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(AppError::InvalidInput)
}

fn index(value: &Value) -> AppResult<u32> {
    value
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or(AppError::InvalidInput)
}

fn optional_text(value: &Value, snake: &str, camel: &str, web: bool) -> AppResult<String> {
    if value.get(key(camel, snake, web)).is_some() {
        return Err(AppError::InvalidInput);
    }

    match value.get(key(snake, camel, web)) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(value) => text(value),
    }
}

fn parse_unit(value: &Value, page_id: &str, page_index: u32, web: bool) -> AppResult<ArchiveUnit> {
    text(field(value, "unit_id", "unitId", web)?)?;

    if text(field(value, "page_id", "pageId", web)?)? != page_id
        || index(field(value, "page_index", "pageIndex", web)?)? != page_index
    {
        return Err(AppError::InvalidInput);
    }

    if value.get(key("isFlagged", "is_flagged", web)).is_some() {
        return Err(AppError::InvalidInput);
    }

    let flag = match value.get(key("is_flagged", "isFlagged", web)) {
        None => false,
        Some(value) => value.as_bool().ok_or(AppError::InvalidInput)?,
    };

    Ok(ArchiveUnit {
        index: index(field(value, "unit_index", "unitIndex", web)?)?,
        x_coord: field(value, "x_coord", "xCoord", web)?
            .as_f64()
            .ok_or(AppError::InvalidInput)?,
        y_coord: field(value, "y_coord", "yCoord", web)?
            .as_f64()
            .ok_or(AppError::InvalidInput)?,
        is_bubble: field(value, "is_bubble", "isBubble", web)?
            .as_bool()
            .ok_or(AppError::InvalidInput)?,
        is_flagged: flag,
        translated_text: optional_text(value, "translated_text", "translatedText", web)?,
        proofread_text: optional_text(value, "proofread_text", "proofreadText", web)?,
        is_proofread: field(value, "is_proofread", "isProofread", web)?
            .as_bool()
            .ok_or(AppError::InvalidInput)?,
    })
}

/// # Errors
/// Parses one historical naming convention without alias fallback.
pub fn parse(value: &Value, web: bool) -> AppResult<ArchiveDocument> {
    text(field(value, "comic_id", "comicId", web)?)?;

    text(field(value, "chapter_id", "chapterId", web)?)?;

    index(field(value, "chapter_index", "chapterIndex", web)?)?;

    let title = text(field(value, "comic_title", "comicTitle", web)?)?
        .trim()
        .to_owned();

    if title.is_empty() {
        return Err(AppError::InvalidInput);
    }

    let subtitle = optional_text(value, "chapter_subtitle", "chapterSubtitle", web)?
        .trim()
        .to_owned();

    let raw_pages = value["pages"].as_array().ok_or(AppError::InvalidInput)?;

    let mut pages = Vec::new();

    let mut ids = std::collections::HashSet::new();

    let mut missing_flag = false;

    for raw_page in raw_pages {
        let page_id = text(field(raw_page, "page_id", "pageId", web)?)?;

        let page_index = index(field(raw_page, "page_index", "pageIndex", web)?)?;

        if !ids.insert(page_id.clone()) {
            return Err(AppError::InvalidInput);
        }

        let raw_units = raw_page["units"].as_array().ok_or(AppError::InvalidInput)?;

        let mut units = Vec::new();

        for unit in raw_units {
            missing_flag |= unit.get(key("is_flagged", "isFlagged", web)).is_none();

            units.push(parse_unit(unit, &page_id, page_index, web)?);
        }

        normalize_units(&mut units, false)?;

        let source_image_path = match raw_page.get("exportedImagePath") {
            None | Some(Value::Null) => None,
            Some(value) => {
                let path = text(value)?;

                logical_path(&path)?;

                Some(path)
            }
        };

        pages.push(ArchivePage {
            index: page_index,
            image: None,
            source_image_path,
            units,
        });
    }

    pages.sort_by_key(|page| page.index);

    for (index, page) in pages.iter().enumerate() {
        if page.index != position(index)? {
            return Err(AppError::InvalidInput);
        }
    }

    let mut warnings =
        vec!["历史身份、贡献者及其他非本地字段不予保留；请核对图片方向与标记".to_owned()];

    if missing_flag {
        warnings.push("来源缺少关注信息，已设为未关注".to_owned());
    }

    Ok(ArchiveDocument {
        comic: ArchiveComic {
            title,
            subtitle,
            author: String::new(),
        },
        pages,
        warnings,
    })
}
