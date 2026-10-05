use std::fs;
use std::path::PathBuf;

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use uuid::Uuid;

use crate::implementation::image::ImagePipeline;
use crate::implementation::image::decode::{check_dimensions, decode};
use crate::implementation::image::error::ImageError;
use crate::implementation::resource::ResourceStore;

fn fixture_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("poprako-image-test-{}", Uuid::new_v4()));

    fs::create_dir(&root)?;

    Ok(root)
}

#[test]
fn actual_static_formats_and_lossless_tiles() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root()?;

    for format in [
        ImageFormat::Jpeg,
        ImageFormat::Png,
        ImageFormat::WebP,
        ImageFormat::Bmp,
    ] {
        let path = root.join(format!("{format:?}.wrong-extension"));

        let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(8, 12, |x, y| {
            Rgba([
                u8::try_from(x).unwrap_or(0),
                u8::try_from(y).unwrap_or(0),
                77,
                255,
            ])
        }));

        image.to_rgb8().save_with_format(&path, format)?;

        let decoded = decode(&path)?;

        let tile = image::load_from_memory(&decoded.tile(1, 2, 4, 5)?)?.into_rgba8();

        assert_eq!(decoded.format, format);

        assert_eq!(tile.get_pixel(0, 0), decoded.pixels.get_pixel(1, 2));

        assert!(matches!(
            decoded.tile(7, 0, 2, 1),
            Err(ImageError::InvalidRegion)
        ));

        assert_eq!(image::load_from_memory(&decoded.preview(true)?)?.width(), 8);
    }

    fs::remove_dir_all(root)?;

    Ok(())
}

#[test]
fn dimension_and_truncation_protection() -> Result<(), Box<dyn std::error::Error>> {
    assert!(check_dimensions(8000, 4000).is_ok());

    assert!(check_dimensions(16384, 1).is_ok());

    assert!(matches!(
        check_dimensions(8001, 4000),
        Err(ImageError::Dimensions)
    ));

    assert!(matches!(
        check_dimensions(16385, 1),
        Err(ImageError::Dimensions)
    ));

    let root = fixture_root()?;

    let path = root.join("truncated.png");

    fs::write(&path, b"\x89PNG\r\n\x1a\n")?;

    assert!(decode(&path).is_err());

    fs::remove_dir_all(root)?;

    Ok(())
}

#[test]
fn exif_orientation_changes_display_not_original() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root()?;

    let path = root.join("oriented.jpg");

    DynamicImage::new_rgb8(8, 12).save_with_format(&path, ImageFormat::Jpeg)?;

    let original = fs::read(&path)?;

    let exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";

    let mut bytes = original[..2].to_vec();

    bytes.extend_from_slice(&[0xff, 0xe1]);

    bytes.extend_from_slice(&u16::try_from(exif.len() + 2)?.to_be_bytes());

    bytes.extend_from_slice(exif);

    bytes.extend_from_slice(&original[2..]);

    fs::write(&path, &bytes)?;

    let decoded = decode(&path)?;

    assert_eq!(decoded.pixels.dimensions(), (12, 8));

    assert_eq!(fs::read(&path)?, bytes);

    fs::remove_dir_all(root)?;

    Ok(())
}

#[test]
fn managed_publication_preserves_bytes_and_rejects_escape() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root()?;

    let source = root.join("source.png");

    DynamicImage::new_rgba8(8, 12).save(&source)?;

    let original = fs::read(&source)?;

    let store = ResourceStore::open(&root.join("managed"))?;

    let pipeline = ImagePipeline::default();

    let prepared = store.prepare(&source, Uuid::new_v4(), &pipeline)?;

    let comic_id = Uuid::new_v4();

    let page_id = Uuid::new_v4();

    let reference = store.publish(&prepared, comic_id, page_id)?;

    assert_eq!(
        fs::read(store.resolve(&reference, comic_id, page_id)?)?,
        original
    );

    assert!(store.publish(&prepared, comic_id, page_id).is_err());

    assert!(
        store
            .resolve("../../source.png", comic_id, page_id)
            .is_err()
    );

    assert!(store.resolve(&reference, Uuid::new_v4(), page_id).is_err());

    store.discard_prepared(&prepared)?;

    pipeline.release()?;

    fs::remove_dir_all(root)?;

    Ok(())
}

#[test]
fn truncated_pixel_streams_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root()?;

    for format in [
        ImageFormat::Jpeg,
        ImageFormat::Png,
        ImageFormat::WebP,
        ImageFormat::Bmp,
    ] {
        let path = root.join(format!("truncated-{format:?}"));

        DynamicImage::new_rgb8(80, 120).save_with_format(&path, format)?;

        let bytes = fs::read(&path)?;

        fs::write(&path, &bytes[..bytes.len() / 2])?;

        assert!(decode(&path).is_err(), "{format:?}");
    }

    fs::remove_dir_all(root)?;

    Ok(())
}
