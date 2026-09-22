pub mod decoder;
pub mod encoder;
pub mod error;
pub mod formats;
pub mod options;
pub mod svg;
pub mod transformer;

pub use error::ConversionError;
pub use formats::{ImageFormat, InputFormat};
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
    let image = match InputFormat::from_path(input_path) {
        Some(InputFormat::Svg) => svg::decode_path(input_path, preferred_render_size(options))?,
        _ => decoder::decode_path(input_path)?.0,
    };
    finish(image, output_path, format, options)
}

pub fn convert_bytes(
    data: &[u8],
    output_path: &Path,
    format: ImageFormat,
    options: &ConversionOptions,
) -> Result<ConvertedImage, ConversionError> {
    let image = if svg::looks_like_svg(data) {
        svg::decode_bytes(data, preferred_render_size(options))?
    } else {
        decoder::decode_bytes(data)?.0
    };
    finish(image, output_path, format, options)
}

fn preferred_render_size(options: &ConversionOptions) -> Option<svg::RenderSize> {
    let resize = options.transform.resize.as_ref().filter(|r| r.exact)?;
    if options.transform.crop.is_some() {
        return None;
    }
    Some(svg::RenderSize {
        width: resize.width,
        height: resize.height,
    })
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
