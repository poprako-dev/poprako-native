use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use uuid::Uuid;

use crate::complex::archive::json::{METADATA_LIMIT, parse_prk};
use crate::complex::archive::label_plus;
use crate::complex::archive::path::logical_path;
use crate::data::archive::ArchiveDocument;
use crate::implementation::archive::package::{copy_entry, open, read_document};
use crate::implementation::image::ImagePipeline;
use crate::implementation::resource::{PreparedImage, ResourceStore};
use crate::result::{AppError, AppResult};

pub struct PreparedArchive {
    pub document: ArchiveDocument,
    pub images: Vec<PreparedImage>,
    pub image_names: Vec<String>,
}

fn copy_snapshot(
    source: &mut File,
    destination: &mut File,
    progress: &mut impl FnMut(usize, usize) -> AppResult<()>,
) -> AppResult<()> {
    let mut buffer = vec![0_u8; 64 * 1024];

    loop {
        progress(0, 0)?;

        let length = source.read(&mut buffer).map_err(|_| AppError::Storage)?;

        if length == 0 {
            return Ok(());
        }

        destination
            .write_all(&buffer[..length])
            .map_err(|_| AppError::Storage)?;
    }
}

fn parse_text(source: &Path, stage: &crate::data::search::TextStage) -> AppResult<ArchiveDocument> {
    let file = File::open(source).map_err(|_| AppError::Storage)?;

    let mut bytes = Vec::new();

    file.take(METADATA_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AppError::Storage)?;

    match source
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        true => parse_prk(&bytes),
        false => label_plus::parse_for_stage(&bytes, stage),
    }
}

fn prepare_images(
    document: ArchiveDocument,
    paths: &[PathBuf],
    store: &ResourceStore,
    pipeline: &ImagePipeline,
    progress: &mut impl FnMut(usize, usize) -> AppResult<()>,
) -> AppResult<PreparedArchive> {
    if paths.len() != document.pages.len() {
        return Err(AppError::InvalidInput);
    }

    let task_id = Uuid::new_v4();

    let mut images = Vec::new();

    for (page, path) in document.pages.iter().zip(paths) {
        let result = progress(images.len(), paths.len()).and_then(|()| {
            store
                .prepare(path, task_id, pipeline)
                .map_err(|_| AppError::InvalidInput)
        });

        let prepared = match result {
            Ok(image) => image,
            Err(error) => {
                for image in &images {
                    store
                        .discard_prepared(image)
                        .map_err(|_| AppError::Storage)?;
                }

                return Err(error);
            }
        };

        let mismatch = page.image.as_ref().is_some_and(|expected| {
            let extension = match expected.format {
                crate::value::image::ImageFormat::Jpeg => "jpg",
                crate::value::image::ImageFormat::Png => "png",
                crate::value::image::ImageFormat::Webp => "webp",
                crate::value::image::ImageFormat::Bmp => "bmp",
            };

            expected.width != prepared.width
                || expected.height != prepared.height
                || extension != prepared.extension
        });

        images.push(prepared);

        if mismatch {
            for image in &images {
                store
                    .discard_prepared(image)
                    .map_err(|_| AppError::Storage)?;
            }

            return Err(AppError::InvalidInput);
        }
    }

    let image_names = paths
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned()
        })
        .collect();

    Ok(PreparedArchive {
        document,
        images,
        image_names,
    })
}

/// # Errors
/// Only accepts paths retained by the native picker authorization layer.
/// External images are an explicitly ordered, complete replacement pairing.
pub fn prepare_controlled(
    source: &Path,
    external_images: &[PathBuf],
    staging_parent: &Path,
    store: &ResourceStore,
    pipeline: &ImagePipeline,
    stage: &crate::data::search::TextStage,
    progress: &mut impl FnMut(usize, usize) -> AppResult<()>,
) -> AppResult<PreparedArchive> {
    if !source
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        return prepare_images(
            parse_text(source, stage)?,
            external_images,
            store,
            pipeline,
            progress,
        );
    }

    let staging = TempDir::new_in(staging_parent).map_err(|_| AppError::Storage)?;

    let mut snapshot = tempfile::tempfile_in(staging.path()).map_err(|_| AppError::Storage)?;

    let mut original = File::open(source).map_err(|_| AppError::Storage)?;

    copy_snapshot(&mut original, &mut snapshot, progress)?;

    snapshot.sync_all().map_err(|_| AppError::Storage)?;

    let mut package = open(snapshot)?;

    let document = read_document(&mut package, stage)?;

    if !external_images.is_empty() {
        return prepare_images(document, external_images, store, pipeline, progress);
    }

    let mut paths = Vec::new();

    for page in &document.pages {
        progress(paths.len(), document.pages.len())?;

        let path = page
            .source_image_path
            .as_ref()
            .ok_or(AppError::InvalidInput)?;

        let key = logical_path(path)?;

        let nested_key = logical_path(&format!("images/{path}"))?;

        let candidates = package
            .entries
            .iter()
            .filter(|entry| {
                logical_path(&entry.name)
                    .is_ok_and(|entry_key| entry_key == key || entry_key == nested_key)
            })
            .collect::<Vec<_>>();

        if candidates.len() != 1 {
            return Err(AppError::InvalidInput);
        }

        let index = candidates[0].index;

        let size = candidates[0].size;

        let path = staging.path().join(Uuid::new_v4().to_string());

        let mut file = File::create_new(&path).map_err(|_| AppError::Storage)?;

        copy_entry(&mut package, index, &mut file, size)?;

        file.sync_all().map_err(|_| AppError::Storage)?;

        paths.push(path);
    }

    let names = document
        .pages
        .iter()
        .map(|page| {
            page.source_image_path
                .as_deref()
                .and_then(|path| path.rsplit('/').next())
                .unwrap_or_default()
                .to_owned()
        })
        .collect();

    let result = prepare_images(document, &paths, store, pipeline, progress).map(|mut prepared| {
        prepared.image_names = names;

        prepared
    });

    // Windows requires the snapshot handle to close before removing its directory.
    drop(package);

    staging.close().map_err(|_| AppError::Storage)?;

    result
}

/// # Errors
/// Reads an owned translation snapshot for an existing fully imaged project.
/// The caller must validate all target images and bind the target page baseline.
pub fn read_translation(
    source: &Path,
    stage: &crate::data::search::TextStage,
) -> AppResult<ArchiveDocument> {
    if source
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        let mut package = open(File::open(source).map_err(|_| AppError::Storage)?)?;

        return read_document(&mut package, stage);
    }

    parse_text(source, stage)
}

/// # Errors
/// Prepares a complete archive without a task progress observer.
pub fn prepare(
    source: &Path,
    images: &[PathBuf],
    staging: &Path,
    store: &ResourceStore,
    pipeline: &ImagePipeline,
    stage: &crate::data::search::TextStage,
) -> AppResult<PreparedArchive> {
    prepare_controlled(
        source,
        images,
        staging,
        store,
        pipeline,
        stage,
        &mut |_, _| Ok(()),
    )
}
