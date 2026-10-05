use std::fmt;

#[derive(Debug)]
pub enum ImageError {
    Io(std::io::Error),
    Decode(image::ImageError),
    Unsupported,
    Animated,
    Dimensions,
    Budget,
    InvalidRegion,
    InvalidReference,
    Unavailable,
    Cancelled,
}

impl fmt::Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Io(_) => "无法读取或保存图片，请检查文件和剩余空间",
            Self::Decode(_) => "图片损坏或编码不受支持，请更换图片",
            Self::Unsupported => "仅支持静态 JPEG、PNG、WebP 和 BMP 图片",
            Self::Animated => "不支持动画，请先导出静态图片",
            Self::Dimensions => "图片超过 3200 万像素或最长边 16384，请先缩小图片",
            Self::Budget => "图片解码需要的内存超过预算，请先缩小图片",
            Self::InvalidRegion => "请求的图片区域无效",
            Self::InvalidReference => "图片引用无效，请重新打开项目",
            Self::Cancelled => "已取消图片准备",
            Self::Unavailable => "图片处理暂时不可用，请重新启动应用",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ImageError {}

impl From<std::io::Error> for ImageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(error: image::ImageError) -> Self {
        Self::Decode(error)
    }
}
