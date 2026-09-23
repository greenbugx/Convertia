use serde::{Deserialize, Serialize};

use super::vectorizer_config::VectorizationOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PngCompression {
    Fast,
    #[default]
    Balanced,
    Maximum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CropOptions {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResizeOptions {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub exact: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TransformOptions {
    pub rotation: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub crop: Option<CropOptions>,
    pub resize: Option<ResizeOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EncodeOptions {
    pub jpeg_quality: Option<u8>,
    pub png_compression: Option<PngCompression>,
    pub webp_quality: Option<u8>,
    pub avif_quality: Option<u8>,
    pub avif_speed: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ConversionOptions {
    pub transform: TransformOptions,
    pub encode: EncodeOptions,
    pub vectorize: VectorizationOptions,
}
