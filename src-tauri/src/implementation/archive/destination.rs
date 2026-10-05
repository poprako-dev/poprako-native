use std::io::Read;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::data::archive::ArchiveDocument;
use crate::implementation::archive::package::{open, read_document, write_package};
use crate::result::{AppError, AppResult};

/// # Errors
/// Target and immutable source readers must originate from authorized native state.
/// Publishing over an existing file requires explicit confirmation from the caller.
pub fn export_controlled<F, R>(
    target: &Path,
    document: &ArchiveDocument,
    include_images: bool,
    overwrite_confirmed: bool,
    image_reader: F,
    before_publish: impl FnOnce() -> AppResult<()>,
) -> AppResult<()>
where
    F: FnMut(usize) -> AppResult<R>,
    R: Read,
{
    let parent = target.parent().ok_or(AppError::InvalidInput)?;

    let mut temporary = NamedTempFile::new_in(parent).map_err(|_| AppError::Storage)?;

    write_package(
        temporary.as_file_mut(),
        document,
        include_images,
        image_reader,
    )?;

    temporary
        .as_file()
        .sync_all()
        .map_err(|_| AppError::Storage)?;

    let file = temporary.reopen().map_err(|_| AppError::Storage)?;

    let mut verification = open(file)?;

    read_document(
        &mut verification,
        &crate::data::search::TextStage::Translation,
    )?;

    before_publish()?;

    match overwrite_confirmed {
        true => temporary.persist(target).map_err(|_| AppError::Storage)?,
        false => temporary
            .persist_noclobber(target)
            .map_err(|_| AppError::Conflict)?,
    };

    Ok(())
}

/// # Errors
/// Exports to a native-authorized destination without a task observer.
pub fn export<F, R>(
    target: &Path,
    document: &ArchiveDocument,
    include_images: bool,
    overwrite_confirmed: bool,
    image_reader: F,
) -> AppResult<()>
where
    F: FnMut(usize) -> AppResult<R>,
    R: Read,
{
    export_controlled(
        target,
        document,
        include_images,
        overwrite_confirmed,
        image_reader,
        || Ok(()),
    )
}
