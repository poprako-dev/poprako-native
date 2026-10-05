pub mod decode;
pub mod display;
pub mod error;

use std::path::Path;
use std::sync::Mutex;

use crate::implementation::image::decode::decode;
use crate::implementation::image::display::DecodedImage;
use crate::implementation::image::error::ImageError;

/// Serializes complete decodes and bounds resident full-resolution pixels to one image.
#[derive(Default)]
pub struct ImagePipeline {
    current: Mutex<Option<DecodedImage>>,
}

impl ImagePipeline {
    /// Runs blocking codec work. Call from a blocking worker, never the UI executor.
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn with_image<T>(
        &self,
        path: &Path,
        consume: impl FnOnce(&DecodedImage) -> Result<T, ImageError>,
    ) -> Result<T, ImageError> {
        let mut current = self.current.lock().map_err(|_| ImageError::Unavailable)?;

        if current.as_ref().is_none_or(|image| image.path != path) {
            *current = None;

            *current = Some(decode(path)?);
        }

        let image = current.as_ref().ok_or(ImageError::Unavailable)?;

        consume(image)
    }

    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn release(&self) -> Result<(), ImageError> {
        *self.current.lock().map_err(|_| ImageError::Unavailable)? = None;

        Ok(())
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod animation_test;

#[cfg(test)]
mod budget_test;

#[cfg(test)]
mod orientation_test;
