use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Bmp,
}

impl ImageFormat {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Bmp => "bmp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Image {
    pub reference: String,
    pub original_name: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
}
