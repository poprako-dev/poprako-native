use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use image::codecs::bmp::BmpDecoder;
use image::codecs::jpeg::JpegDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};

use crate::implementation::image::display::DecodedImage;
use crate::implementation::image::error::ImageError;

pub const WORKSPACE_BYTES: u64 = 768 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 32_000_000;
pub const MAX_EDGE: u32 = 16_384;

/// # Errors
/// Returns an error when validation, resource access, or image processing fails.
pub fn check_dimensions(width: u32, height: u32) -> Result<(), ImageError> {
    if width == 0
        || height == 0
        || width.max(height) > MAX_EDGE
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(ImageError::Dimensions);
    }

    Ok(())
}

fn limits() -> Limits {
    let mut limits = Limits::default();

    limits.max_image_width = Some(MAX_EDGE);

    limits.max_image_height = Some(MAX_EDGE);

    limits.max_alloc = Some(WORKSPACE_BYTES);

    limits
}

/// # Errors
/// Returns an error when validation, resource access, or image processing fails.
pub fn extension(format: ImageFormat) -> Result<&'static str, ImageError> {
    match format {
        ImageFormat::Jpeg => Ok("jpg"),
        ImageFormat::Png => Ok("png"),
        ImageFormat::WebP => Ok("webp"),
        ImageFormat::Bmp => Ok("bmp"),
        _ => Err(ImageError::Unsupported),
    }
}

fn decoder(path: &Path, format: ImageFormat) -> Result<Box<dyn ImageDecoder>, ImageError> {
    let reader = BufReader::new(File::open(path)?);

    match format {
        ImageFormat::Jpeg => Ok(Box::new(JpegDecoder::new(reader)?)),
        ImageFormat::Bmp => Ok(Box::new(BmpDecoder::new(reader)?)),
        ImageFormat::Png => {
            let mut header_limits = limits();

            // PNG metadata and internal buffers have a separate bounded allocation pool.
            header_limits.max_alloc = Some(64 * 1024 * 1024);

            let decoder = PngDecoder::with_limits(reader, header_limits)?;

            if decoder.is_apng()? {
                return Err(ImageError::Animated);
            }

            Ok(Box::new(decoder))
        }
        ImageFormat::WebP => {
            let decoder = WebPDecoder::new(reader)?;

            if decoder.has_animation() {
                return Err(ImageError::Animated);
            }

            Ok(Box::new(decoder))
        }
        _ => Err(ImageError::Unsupported),
    }
}

/// # Errors
/// Returns an error when validation, resource access, or image processing fails.
pub fn decode(path: &Path) -> Result<DecodedImage, ImageError> {
    let input_bytes = path.metadata()?.len();

    // JPEG buffers the input during construction. Bound it before invoking codecs.
    if input_bytes > WORKSPACE_BYTES / 4 {
        return Err(ImageError::Budget);
    }

    let format = ImageReader::open(path)?
        .with_guessed_format()?
        .format()
        .ok_or(ImageError::Unsupported)?;

    extension(format)?;

    let mut decoder = decoder(path, format)?;

    let (width, height) = decoder.dimensions();

    check_dimensions(width, height)?;

    let pixel_bytes = u64::from(width) * u64::from(height) * 4;

    let estimate = input_bytes
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(decoder.total_bytes().checked_mul(2)?))
        .and_then(|bytes| bytes.checked_add(pixel_bytes.checked_mul(2)?))
        .and_then(|bytes| bytes.checked_add(64 * 1024 * 1024))
        .ok_or(ImageError::Budget)?;

    if estimate > WORKSPACE_BYTES {
        return Err(ImageError::Budget);
    }

    // Limits do not provide process isolation; codec workspace must also be measured.
    decoder.set_limits(limits())?;

    let orientation = decoder.orientation()?;

    let mut pixels = DynamicImage::from_decoder(decoder)?;

    pixels.apply_orientation(orientation);

    Ok(DecodedImage {
        path: path.to_path_buf(),
        format,
        pixels: pixels.into_rgba8(),
        input_bytes,
        estimated_workspace_bytes: estimate,
    })
}
