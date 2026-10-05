use std::fs::{self, File};

use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder};
use uuid::Uuid;

use crate::implementation::image::decode::decode;

fn write_oriented(
    encoder: impl ImageEncoder,
    orientation: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = encoder;

    let mut metadata =
        b"II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x01\0\0\0\0\0\0\0".to_vec();

    metadata[18] = orientation;

    encoder.set_exif_metadata(metadata)?;

    let pixels = [0, 1, 2, 3, 4, 5].map(|red| [red, 20, 30, 255]).concat();

    encoder.write_image(&pixels, 2, 3, ExtendedColorType::Rgba8)?;

    Ok(())
}

#[test]
fn png_and_webp_exif_all_eight_orientations() -> Result<(), Box<dyn std::error::Error>> {
    let expected = [
        [0, 1, 2, 3, 4, 5],
        [1, 0, 3, 2, 5, 4],
        [5, 4, 3, 2, 1, 0],
        [4, 5, 2, 3, 0, 1],
        [0, 2, 4, 1, 3, 5],
        [4, 2, 0, 5, 3, 1],
        [5, 3, 1, 4, 2, 0],
        [1, 3, 5, 0, 2, 4],
    ];

    for (index, expected) in expected.iter().enumerate() {
        let orientation = u8::try_from(index + 1)?;

        for extension in ["png", "webp"] {
            let path =
                std::env::temp_dir().join(format!("poprako-exif-{}.{}", Uuid::new_v4(), extension));

            let output = File::create(&path)?;

            match extension {
                "png" => write_oriented(PngEncoder::new(output), orientation)?,
                _ => write_oriented(WebPEncoder::new_lossless(output), orientation)?,
            }

            let image = decode(&path)?;

            let actual: Vec<_> = image.pixels.pixels().map(|pixel| pixel[0]).collect();

            assert_eq!(&actual, expected, "{extension} orientation {orientation}");

            fs::remove_file(path)?;
        }
    }

    Ok(())
}
