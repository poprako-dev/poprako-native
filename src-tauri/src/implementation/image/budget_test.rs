use std::fs;

use image::{DynamicImage, ImageFormat};
use uuid::Uuid;

use crate::implementation::image::decode::{WORKSPACE_BYTES, decode};
use crate::implementation::image::error::ImageError;

#[test]
fn real_32_megapixel_image_over_50_mib_decodes_and_tiles() -> Result<(), Box<dyn std::error::Error>>
{
    let path = std::env::temp_dir().join(format!("poprako-budget-{}.bmp", Uuid::new_v4()));

    DynamicImage::new_rgb8(8000, 4000).save_with_format(&path, ImageFormat::Bmp)?;

    let image = decode(&path)?;

    assert!(image.input_bytes > 50 * 1024 * 1024);

    assert!(image.estimated_workspace_bytes <= WORKSPACE_BYTES);

    assert_eq!(image.pixels.dimensions(), (8000, 4000));

    assert_eq!(image.pixels.as_raw().len(), 128_000_000);

    assert_eq!(
        image::load_from_memory(&image.tile(6976, 2976, 1024, 1024)?)?.width(),
        1024
    );

    assert_eq!(
        image::load_from_memory(&image.preview(false)?)?.width(),
        2048
    );

    drop(image);

    // Header-only modification exercises real codec metadata rejection before allocation.
    let mut bytes = fs::read(&path)?;

    bytes[18..22].copy_from_slice(&8001_i32.to_le_bytes());

    fs::write(&path, bytes)?;

    assert!(matches!(decode(&path), Err(ImageError::Dimensions)));

    fs::remove_file(path)?;

    Ok(())
}
