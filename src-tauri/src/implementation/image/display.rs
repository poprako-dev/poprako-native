use std::path::PathBuf;

use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage};

use crate::implementation::image::error::ImageError;

fn encode(pixels: &RgbaImage) -> Result<Vec<u8>, ImageError> {
    let mut output = Vec::new();

    WebPEncoder::new_lossless(&mut output).write_image(
        pixels.as_raw(),
        pixels.width(),
        pixels.height(),
        ExtendedColorType::Rgba8,
    )?;

    Ok(output)
}

pub struct DecodedImage {
    pub path: PathBuf,
    pub format: ImageFormat,
    pub pixels: RgbaImage,
    pub input_bytes: u64,
    pub estimated_workspace_bytes: u64,
}

impl DecodedImage {
    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn preview(&self, thumbnail: bool) -> Result<Vec<u8>, ImageError> {
        let edge = [2048_u32, 256_u32][usize::from(thumbnail)];

        let longest = self.pixels.width().max(self.pixels.height()).max(edge);

        let width = (self.pixels.width() * edge / longest).max(1);

        let height = (self.pixels.height() * edge / longest).max(1);

        let pixels = image::imageops::resize(&self.pixels, width, height, FilterType::Triangle);

        encode(&pixels)
    }

    /// # Errors
    /// Returns an error when validation, resource access, or image processing fails.
    pub fn tile(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
        if width == 0
            || height == 0
            || width > 1024
            || height > 1024
            || x.checked_add(width)
                .is_none_or(|end| end > self.pixels.width())
            || y.checked_add(height)
                .is_none_or(|end| end > self.pixels.height())
        {
            return Err(ImageError::InvalidRegion);
        }

        encode(&image::imageops::crop_imm(&self.pixels, x, y, width, height).to_image())
    }
}
