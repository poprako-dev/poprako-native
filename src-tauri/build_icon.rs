use std::fs;
use std::io;
use std::path::Path;

fn write_changed(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(());
    }

    fs::write(path, bytes)
}

/// # Errors
/// Reports an invalid source PNG or failure to write the platform containers.
pub fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let png = fs::read("icons/icon.png")?;

    if png.get(..8) != Some(b"\x89PNG\r\n\x1a\n")
        || png.get(12..16) != Some(b"IHDR")
        || png.get(16..24) != Some(&[0, 0, 1, 0, 0, 0, 1, 0])
    {
        return Err(io::Error::other("icons/icon.png must be a 256x256 PNG").into());
    }

    let size = u32::try_from(png.len())?;

    let chunk_size = size
        .checked_add(8)
        .ok_or_else(|| io::Error::other("Icon is too large"))?;

    let container_size = chunk_size
        .checked_add(8)
        .ok_or_else(|| io::Error::other("Icon is too large"))?;

    // ICNS ic08 carries the unchanged 256px PNG; no extra raster sizes are stored.
    let mut icns = Vec::new();

    icns.extend_from_slice(b"icns");

    icns.extend_from_slice(&container_size.to_be_bytes());

    icns.extend_from_slice(b"ic08");

    icns.extend_from_slice(&chunk_size.to_be_bytes());

    icns.extend_from_slice(&png);

    // ICO dimensions of zero encode 256px, with one 32-bit PNG image at offset 22.
    let mut ico = vec![0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 32, 0];

    ico.extend_from_slice(&size.to_le_bytes());

    ico.extend_from_slice(&22_u32.to_le_bytes());

    ico.extend_from_slice(&png);

    let directory = Path::new("target/generated-icon");

    fs::create_dir_all(directory)?;

    write_changed(&directory.join("icon.icns"), &icns)?;

    write_changed(&directory.join("icon.ico"), &ico)?;

    Ok(())
}
