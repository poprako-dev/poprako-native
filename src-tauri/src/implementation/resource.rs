pub mod cache;
pub mod lease;
pub mod recovery;
pub mod registry;

pub mod path;

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use uuid::Uuid;

use crate::implementation::image::ImagePipeline;
use crate::implementation::image::decode::extension;
use crate::implementation::image::error::ImageError;
use crate::implementation::resource::cache::{DisplayCache, DisplayRequest};
use crate::implementation::resource::path::{
    checked_path, cleanup_path, original_reference, valid_id,
};

#[derive(Clone)]
pub struct PreparedImage {
    pub task_id: Uuid,
    pub image_id: Uuid,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    pub byte_length: u64,
    staged_path: PathBuf,
}

pub struct ResourceStore {
    root: PathBuf,
    cache: DisplayCache,
    readers: Arc<lease::OriginalLeaseRegistry>,
}

impl ResourceStore {
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn open(root: &Path) -> Result<Self, ImageError> {
        fs::create_dir_all(root)?;

        let root = root.canonicalize()?;

        for directory in ["library", "staging", "cache"] {
            let path = root.join(directory);

            if !path.exists() {
                fs::create_dir(&path)?;
            }

            checked_path(&root, Path::new(directory))?;
        }

        let cache = DisplayCache::open(&root)?;

        Ok(Self {
            root,
            cache,
            readers: Arc::new(lease::OriginalLeaseRegistry::default()),
        })
    }

    /// Source paths must originate from a Rust-owned file picker authorization.
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn prepare(
        &self,
        source: &Path,
        task_id: Uuid,
        pipeline: &ImagePipeline,
    ) -> Result<PreparedImage, ImageError> {
        self.prepare_with_progress(source, task_id, pipeline, |_| Ok(()))
    }

    /// # Errors
    /// Returns a validation, storage, decode, or cancellation error while streaming a copy.
    pub fn prepare_with_progress(
        &self,
        source: &Path,
        task_id: Uuid,
        pipeline: &ImagePipeline,
        mut progress: impl FnMut(u64) -> Result<(), ImageError>,
    ) -> Result<PreparedImage, ImageError> {
        valid_id(&task_id.to_string())?;

        let relative = PathBuf::from("staging").join(task_id.to_string());

        let task = self.root.join(&relative);

        if !task.exists() {
            fs::create_dir(&task)?;
        }

        checked_path(&self.root, &relative)?;

        let image_id = Uuid::new_v4();

        let staged_path = task.join(image_id.to_string());

        let mut source = File::open(source)?;

        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged_path)?;

        let result = (|| {
            let mut byte_length = 0_u64;

            let mut buffer = vec![0_u8; 1024 * 1024];

            loop {
                let length = source.read(&mut buffer)?;

                if length == 0 {
                    break;
                }

                output.write_all(&buffer[..length])?;

                byte_length = byte_length
                    .checked_add(u64::try_from(length).map_err(|_| ImageError::Budget)?)
                    .ok_or(ImageError::Budget)?;

                if byte_length > crate::implementation::image::decode::WORKSPACE_BYTES / 4 {
                    return Err(ImageError::Budget);
                }

                progress(byte_length)?;
            }

            drop(buffer);

            output.sync_all()?;

            let prepared = pipeline.with_image(&staged_path, |image| {
                Ok(PreparedImage {
                    task_id,
                    image_id,
                    extension: extension(image.format)?.to_owned(),
                    width: image.pixels.width(),
                    height: image.pixels.height(),
                    byte_length,
                    staged_path: staged_path.clone(),
                })
            })?;

            self.cache
                .get(pipeline, &staged_path, image_id, DisplayRequest::Thumbnail)?;

            self.cache
                .get(pipeline, &staged_path, image_id, DisplayRequest::Preview)?;

            Ok(prepared)
        })();

        if result.is_err() {
            fs::remove_file(&staged_path)?;
        }

        result
    }

    /// Publishes without replacement. The caller commits the database reference afterwards.
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn publish(
        &self,
        image: &PreparedImage,
        comic_id: Uuid,
        page_id: Uuid,
    ) -> Result<String, ImageError> {
        valid_id(&comic_id.to_string())?;

        valid_id(&page_id.to_string())?;

        valid_id(&image.image_id.to_string())?;

        valid_id(&image.task_id.to_string())?;

        if !matches!(image.extension.as_str(), "jpg" | "png" | "webp" | "bmp") {
            return Err(ImageError::InvalidReference);
        }

        let staged = PathBuf::from("staging")
            .join(image.task_id.to_string())
            .join(image.image_id.to_string());

        let source = checked_path(&self.root, &staged)?;

        if source != image.staged_path {
            return Err(ImageError::InvalidReference);
        }

        let mut directory = PathBuf::from("library");

        for id in [comic_id, page_id] {
            directory.push(id.to_string());

            let path = self.root.join(&directory);

            if !path.exists() {
                fs::create_dir(&path)?;
            }

            checked_path(&self.root, &directory)?;
        }

        let filename = format!("{}.{}", image.image_id, image.extension);

        let relative = directory.join(filename);

        // A hard link on the same volume provides no-replace publication.
        fs::hard_link(source, self.root.join(&relative))?;

        Ok(relative.to_string_lossy().replace('\\', "/"))
    }

    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn resolve(
        &self,
        reference: &str,
        comic_id: Uuid,
        page_id: Uuid,
    ) -> Result<PathBuf, ImageError> {
        let relative = original_reference(reference, comic_id, page_id)?;

        checked_path(&self.root, &relative)
    }

    /// Returns only generated display resources for a validated immutable reference.
    /// # Errors
    /// Returns an error for invalid ownership, unavailable originals, or invalid regions.
    pub fn display(
        &self,
        reference: &str,
        comic_id: Uuid,
        page_id: Uuid,
        request: DisplayRequest,
        pipeline: &ImagePipeline,
    ) -> Result<Vec<u8>, ImageError> {
        let original = self.resolve(reference, comic_id, page_id)?;

        let image_id = original
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(ImageError::InvalidReference)?;

        self.cache
            .get(pipeline, &original, valid_id(image_id)?, request)
    }

    /// # Errors
    /// Returns an error if the original is unavailable or resource ownership is invalid.
    pub fn open_original(
        &self,
        reference: &str,
        comic_id: Uuid,
        page_id: Uuid,
    ) -> Result<lease::OriginalRead, ImageError> {
        let path = self.resolve(reference, comic_id, page_id)?;

        self.readers.acquire(reference, || Ok(File::open(path)?))
    }

    /// # Errors
    /// Returns an error if the task no longer owns the prepared file.
    pub fn prepared_preview(
        &self,
        prepared: &PreparedImage,
        pipeline: &ImagePipeline,
    ) -> Result<Vec<u8>, ImageError> {
        let relative = PathBuf::from("staging")
            .join(prepared.task_id.to_string())
            .join(prepared.image_id.to_string());

        let path = checked_path(&self.root, &relative)?;

        if path != prepared.staged_path {
            return Err(ImageError::InvalidReference);
        }

        self.cache
            .get(pipeline, &path, prepared.image_id, DisplayRequest::Preview)
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Only use after the caller proves that no database or active reader references remain.
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn remove_unreferenced(
        &self,
        reference: &str,
        comic_id: Uuid,
        page_id: Uuid,
    ) -> Result<(), ImageError> {
        let relative = original_reference(reference, comic_id, page_id)?;

        self.readers.remove(reference, || {
            let Some(path) = cleanup_path(&self.root, &relative)? else {
                return Ok(());
            };

            match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.into()),
            }
        })
    }

    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn discard_prepared(&self, image: &PreparedImage) -> Result<(), ImageError> {
        let relative = PathBuf::from("staging")
            .join(image.task_id.to_string())
            .join(image.image_id.to_string());

        if !self.root.join(&relative).try_exists()? {
            return Ok(());
        }

        let path = checked_path(&self.root, &relative)?;

        if path != image.staged_path {
            return Err(ImageError::InvalidReference);
        }

        fs::remove_file(path)?;

        Ok(())
    }
}

#[cfg(test)]
mod cache_test;
