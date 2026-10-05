use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use image::{ImageFormat, ImageReader};
use uuid::Uuid;

use crate::implementation::image::ImagePipeline;
use crate::implementation::image::error::ImageError;
use crate::implementation::resource::path::{checked_path, valid_id};

const CACHE_BUDGET: u64 = 1024 * 1024 * 1024;
const MAX_ENCODED_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Clone, Copy)]
pub enum DisplayRequest {
    Thumbnail,
    Preview,
    Tile {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

impl DisplayRequest {
    fn key(self) -> String {
        match self {
            Self::Thumbnail => "thumbnail".to_owned(),
            Self::Preview => "preview".to_owned(),
            Self::Tile {
                x,
                y,
                width,
                height,
            } => format!("tile-{x}-{y}-{width}-{height}"),
        }
    }
}

fn read_valid(path: &Path) -> Result<Vec<u8>, ImageError> {
    let mut file = File::open(path)?;

    if file.metadata()?.len() > MAX_ENCODED_BYTES {
        return Err(ImageError::Budget);
    }

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes)?;

    let reader = ImageReader::with_format(Cursor::new(&bytes), ImageFormat::WebP);

    let (width, height) = reader.into_dimensions()?;

    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return Err(ImageError::Dimensions);
    }

    image::load_from_memory_with_format(&bytes, ImageFormat::WebP)?;

    Ok(bytes)
}

fn owned_name(name: &str) -> bool {
    let Some(name) = name
        .strip_prefix("v1-")
        .and_then(|name| name.strip_suffix(".webp"))
    else {
        return false;
    };

    let Some(id) = name.get(..36) else {
        return false;
    };

    let Some(kind) = name.get(37..) else {
        return false;
    };

    if valid_id(id).is_err() || name.as_bytes().get(36) != Some(&b'-') {
        return false;
    }

    if matches!(kind, "thumbnail" | "preview") {
        return true;
    }

    let Some(region) = kind.strip_prefix("tile-") else {
        return false;
    };

    let region: Vec<_> = region.split('-').collect();

    region.len() == 4 && region.iter().all(|part| part.parse::<u32>().is_ok())
}

fn evict(root: &Path, recent: &HashMap<PathBuf, SystemTime>) -> Result<(), ImageError> {
    let mut entries = Vec::new();

    let mut total = 0_u64;

    for entry in fs::read_dir(root)? {
        let entry = entry?;

        let name = entry.file_name();

        let Some(name) = name.to_str() else { continue };

        // Unknown files and links are never part of application cache cleanup.
        if !owned_name(name) || !entry.file_type()?.is_file() {
            continue;
        }

        let metadata = entry.metadata()?;

        let path = entry.path();

        let accessed = recent.get(&path).copied().unwrap_or(metadata.modified()?);

        total = total.saturating_add(metadata.len());

        entries.push((accessed, path, metadata.len()));
    }

    entries.sort_by_key(|entry| entry.0);

    for (_, path, length) in entries {
        if total <= CACHE_BUDGET {
            break;
        }

        fs::remove_file(path)?;

        total = total.saturating_sub(length);
    }

    Ok(())
}

pub struct DisplayCache {
    root: PathBuf,
    recent: Mutex<HashMap<PathBuf, SystemTime>>,
}

impl DisplayCache {
    /// # Errors
    /// Returns an error if the managed cache directory cannot be accessed safely.
    pub fn open(application_root: &Path) -> Result<Self, ImageError> {
        let root = checked_path(&application_root.canonicalize()?, Path::new("cache"))?;

        Ok(Self {
            root,
            recent: Mutex::new(HashMap::new()),
        })
    }

    /// The original path and immutable image ID must be resolved by `ResourceStore` first.
    /// # Errors
    /// Returns an error when the image, requested region, or cache storage is invalid.
    pub fn get(
        &self,
        pipeline: &ImagePipeline,
        original: &Path,
        image_id: Uuid,
        request: DisplayRequest,
    ) -> Result<Vec<u8>, ImageError> {
        let mut recent = self.recent.lock().map_err(|_| ImageError::Unavailable)?;

        if recent.len() >= 4096 {
            recent.clear();
        }

        let relative = PathBuf::from(format!("v1-{image_id}-{}.webp", request.key()));

        let path = self.root.join(&relative);

        if path.exists() {
            checked_path(&self.root, &relative)?;

            if let Ok(bytes) = read_valid(&path) {
                recent.insert(path, SystemTime::now());

                return Ok(bytes);
            }

            fs::remove_file(&path)?;
        }

        let bytes = pipeline.with_image(original, |image| match request {
            DisplayRequest::Thumbnail => image.preview(true),
            DisplayRequest::Preview => image.preview(false),
            DisplayRequest::Tile {
                x,
                y,
                width,
                height,
            } => image.tile(x, y, width, height),
        })?;

        let temporary = self.root.join(format!("{}.pending", Uuid::new_v4()));

        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;

        let result = (|| {
            output.write_all(&bytes)?;

            output.sync_all()?;

            fs::hard_link(&temporary, &path)?;

            Ok::<(), ImageError>(())
        })();

        fs::remove_file(&temporary)?;

        result?;

        recent.insert(path, SystemTime::now());

        evict(&self.root, &recent)?;

        recent.retain(|path, _| path.exists());

        Ok(bytes)
    }
}
