use std::fs;

use image::{DynamicImage, ImageFormat};
use uuid::Uuid;

use crate::implementation::image::decode::decode;
use crate::implementation::image::error::ImageError;

fn crc(bytes: &[u8]) -> u32 {
    let mut value = u32::MAX;

    for byte in bytes {
        value ^= u32::from(*byte);

        for _ in 0..8 {
            value = (value >> 1) ^ (0xedb8_8320_u32.wrapping_mul(value & 1));
        }
    }

    !value
}

#[test]
fn apng_animation_control_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("poprako-apng-{}.png", Uuid::new_v4()));

    DynamicImage::new_rgb8(8, 12).save_with_format(&path, ImageFormat::Png)?;

    let original = fs::read(&path)?;

    let mut bytes = original[..33].to_vec();

    let control = b"acTL\0\0\0\x01\0\0\0\0";

    bytes.extend_from_slice(&8_u32.to_be_bytes());

    bytes.extend_from_slice(control);

    bytes.extend_from_slice(&crc(control).to_be_bytes());

    bytes.extend_from_slice(&original[33..]);

    fs::write(&path, bytes)?;

    assert!(matches!(decode(&path), Err(ImageError::Animated)));

    fs::remove_file(path)?;

    Ok(())
}

#[test]
fn animated_webp_header_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("poprako-webp-{}.webp", Uuid::new_v4()));

    DynamicImage::new_rgb8(8, 12).save_with_format(&path, ImageFormat::WebP)?;

    let original = fs::read(&path)?;

    let mut bytes =
        b"RIFF\0\0\0\0WEBPVP8X\x0a\0\0\0\x02\0\0\0\x07\0\0\x0b\0\0ANIM\x06\0\0\0\0\0\0\0\0\0"
            .to_vec();

    let frame_length = u32::try_from(original.len() - 12 + 16)?;

    bytes.extend_from_slice(b"ANMF");

    bytes.extend_from_slice(&frame_length.to_le_bytes());

    bytes.extend_from_slice(b"\0\0\0\0\0\0\x07\0\0\x0b\0\0\x01\0\0\0");

    bytes.extend_from_slice(&original[12..]);

    let size = u32::try_from(bytes.len() - 8)?;

    bytes[4..8].copy_from_slice(&size.to_le_bytes());

    fs::write(&path, bytes)?;

    assert!(matches!(decode(&path), Err(ImageError::Animated)));

    fs::remove_file(path)?;

    Ok(())
}
