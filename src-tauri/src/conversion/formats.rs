use std::path::Path;

use image::ImageFormat as RasterFormat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Avif,
    Tiff,
    Ico,
    Bmp,
}

impl ImageFormat {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .as_str()
        {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::Webp),
            "avif" => Some(Self::Avif),
            "tif" | "tiff" => Some(Self::Tiff),
            "ico" => Some(Self::Ico),
            "bmp" => Some(Self::Bmp),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    pub fn from_raster(format: RasterFormat) -> Option<Self> {
        match format {
            RasterFormat::Jpeg => Some(Self::Jpeg),
            RasterFormat::Png => Some(Self::Png),
            RasterFormat::WebP => Some(Self::Webp),
            RasterFormat::Avif => Some(Self::Avif),
            RasterFormat::Tiff => Some(Self::Tiff),
            RasterFormat::Ico => Some(Self::Ico),
            RasterFormat::Bmp => Some(Self::Bmp),
            _ => None,
        }
    }

    pub fn raster_format(self) -> RasterFormat {
        match self {
            Self::Jpeg => RasterFormat::Jpeg,
            Self::Png => RasterFormat::Png,
            Self::Webp => RasterFormat::WebP,
            Self::Avif => RasterFormat::Avif,
            Self::Tiff => RasterFormat::Tiff,
            Self::Ico => RasterFormat::Ico,
            Self::Bmp => RasterFormat::Bmp,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Avif => "avif",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Bmp => "bmp",
        }
    }
}

impl Serialize for ImageFormat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.extension())
    }
}

impl<'de> Deserialize<'de> for ImageFormat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        ImageFormat::from_extension(&value)
            .ok_or_else(|| serde::de::Error::custom(format!("unsupported image format: {value}")))
    }
}
