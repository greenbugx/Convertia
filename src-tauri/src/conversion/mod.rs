pub mod decoder;
pub mod encoder;
pub mod error;
pub mod formats;
pub mod options;
pub mod transformer;

pub use error::ConversionError;
pub use formats::ImageFormat;
pub use options::{
    ConversionOptions, CropOptions, EncodeOptions, PngCompression, ResizeOptions, TransformOptions,
};

use std::path::{Path, PathBuf};

use image::DynamicImage;

#[derive(Debug)]
pub struct ConvertedImage {
    pub output_path: PathBuf,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
}

pub fn convert_image(
    input_path: &Path,
    output_path: &Path,
    format: ImageFormat,
    options: &ConversionOptions,
) -> Result<ConvertedImage, ConversionError> {
    let (image, _) = decoder::decode_path(input_path)?;
    finish(image, output_path, format, options)
}

pub fn convert_bytes(
    data: &[u8],
    output_path: &Path,
    format: ImageFormat,
    options: &ConversionOptions,
) -> Result<ConvertedImage, ConversionError> {
    let (image, _) = decoder::decode_bytes(data)?;
    finish(image, output_path, format, options)
}

fn finish(
    image: DynamicImage,
    output_path: &Path,
    format: ImageFormat,
    options: &ConversionOptions,
) -> Result<ConvertedImage, ConversionError> {
    let transformed = transformer::apply(image, &options.transform)?;
    encoder::encode_to_path(&transformed, output_path, format, &options.encode)?;
    Ok(ConvertedImage {
        output_path: output_path.to_path_buf(),
        format,
        width: transformed.width(),
        height: transformed.height(),
    })
}
