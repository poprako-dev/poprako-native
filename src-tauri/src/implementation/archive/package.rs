use std::collections::HashSet;
use std::io::{Read, Seek, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::complex::archive::json::{METADATA_LIMIT, parse_prk};
use crate::complex::archive::label_plus;
use crate::complex::archive::native;
use crate::complex::archive::path::logical_path;
use crate::data::archive::ArchiveDocument;
use crate::result::{AppError, AppResult};

pub struct PackageEntry {
    pub index: usize,
    pub name: String,
    pub size: u64,
}

pub struct Package<R: Read + Seek> {
    archive: ZipArchive<R>,
    pub entries: Vec<PackageEntry>,
}

/// # Errors
/// Rejects ambiguous paths, links, encryption and unchecked directory sizes.
pub fn open<R: Read + Seek>(mut reader: R) -> AppResult<Package<R>> {
    crate::implementation::archive::directory::preflight(&mut reader)?;

    let mut archive = ZipArchive::new(reader).map_err(|_| AppError::InvalidInput)?;

    let mut entries = Vec::new();

    let mut names = HashSet::new();

    let mut metadata_bytes = 0_usize;

    let mut expanded_bytes = 0_u64;

    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| AppError::InvalidInput)?;

        metadata_bytes = metadata_bytes
            .checked_add(entry.name().len())
            .and_then(|count| count.checked_add(128))
            .ok_or(AppError::InvalidInput)?;

        expanded_bytes = expanded_bytes
            .checked_add(entry.size())
            .ok_or(AppError::InvalidInput)?;

        if metadata_bytes > METADATA_LIMIT
            || entry.encrypted()
            || entry.is_symlink()
            || !names.insert(logical_path(entry.name())?)
        {
            return Err(AppError::InvalidInput);
        }

        if entry.is_dir() {
            continue;
        }

        entries.push(PackageEntry {
            index,
            name: entry.name().to_owned(),
            size: entry.size(),
        });
    }

    Ok(Package { archive, entries })
}

/// # Errors
/// Limits actual decompressed bytes and consumes EOF to verify CRC.
pub fn copy_entry<R: Read + Seek, W: Write>(
    package: &mut Package<R>,
    index: usize,
    output: &mut W,
    limit: u64,
) -> AppResult<u64> {
    let mut entry = package
        .archive
        .by_index(index)
        .map_err(|_| AppError::InvalidInput)?;

    let expected = entry.size();

    if expected > limit {
        return Err(AppError::InvalidInput);
    }

    let mut count = 0_u64;

    let mut buffer = vec![0_u8; 64 * 1024];

    loop {
        let length = entry
            .read(&mut buffer)
            .map_err(|_| AppError::InvalidInput)?;

        if length == 0 {
            break;
        }

        count = count
            .checked_add(u64::try_from(length).map_err(|_| AppError::InvalidInput)?)
            .ok_or(AppError::InvalidInput)?;

        if count > limit || count > expected {
            return Err(AppError::InvalidInput);
        }

        output
            .write_all(&buffer[..length])
            .map_err(|_| AppError::Storage)?;
    }

    if count != expected {
        return Err(AppError::InvalidInput);
    }

    Ok(count)
}

/// # Errors
/// A present invalid PRK never falls back to a less complete LP file.
pub fn read_document<R: Read + Seek>(
    package: &mut Package<R>,
    stage: &crate::data::search::TextStage,
) -> AppResult<ArchiveDocument> {
    let roots = package
        .entries
        .iter()
        .filter(|entry| !entry.name.contains('/'))
        .collect::<Vec<_>>();

    let prk = roots
        .iter()
        .filter(|entry| entry.name.to_lowercase().ends_with(".prk.json"))
        .collect::<Vec<_>>();

    let lp = roots
        .iter()
        .filter(|entry| entry.name.to_lowercase().ends_with(".lp.txt"))
        .collect::<Vec<_>>();

    if prk.len() > 1 || lp.len() > 1 {
        return Err(AppError::InvalidInput);
    }

    let (index, is_prk) = match (prk.first(), lp.first()) {
        (Some(entry), _) => (entry.index, true),
        (None, Some(entry)) => (entry.index, false),
        _ => return Err(AppError::InvalidInput),
    };

    let mut bytes = Vec::new();

    copy_entry(package, index, &mut bytes, METADATA_LIMIT as u64)?;

    match is_prk {
        true => parse_prk(&bytes),
        false => label_plus::parse_for_stage(&bytes, stage),
    }
}

/// # Errors
/// Streams immutable original image readers into an unpublished destination.
/// The caller owns destination authorization, synchronization and publication.
pub fn write_package<W, F, R>(
    writer: W,
    document: &ArchiveDocument,
    include_images: bool,
    mut image_reader: F,
) -> AppResult<W>
where
    W: Write + Seek,
    F: FnMut(usize) -> AppResult<R>,
    R: Read,
{
    let prk = native::encode(document)?;

    let lp = label_plus::encode(document)?;

    let mut archive = ZipWriter::new(writer);

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    archive
        .start_file("translation.prk.json", options)
        .map_err(|_| AppError::Storage)?;

    archive.write_all(&prk).map_err(|_| AppError::Storage)?;

    archive
        .start_file("translation.lp.txt", options)
        .map_err(|_| AppError::Storage)?;

    archive.write_all(&lp).map_err(|_| AppError::Storage)?;

    if include_images {
        for (index, page) in document.pages.iter().enumerate() {
            let image = page.image.as_ref().ok_or(AppError::InvalidInput)?;

            let mut reader = image_reader(index)?;

            archive
                .start_file(
                    &image.path,
                    SimpleFileOptions::default()
                        .compression_method(CompressionMethod::Stored)
                        .large_file(true),
                )
                .map_err(|_| AppError::Storage)?;

            std::io::copy(&mut reader, &mut archive).map_err(|_| AppError::Storage)?;
        }
    }

    archive.finish().map_err(|_| AppError::Storage)
}
