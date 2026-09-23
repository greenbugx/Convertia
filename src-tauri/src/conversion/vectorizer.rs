use std::path::Path;

use image::DynamicImage;
use vtracer::{ColorImage, SvgFile};

use super::error::ConversionError;
use super::output;
use super::vectorizer_config::{build_vtracer_config, VectorizationOptions};

pub fn dynamic_image_to_color_image(image: &DynamicImage) -> ColorImage {
    let rgba = image.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    let pixels = rgba.into_raw();
    ColorImage {
        pixels,
        width,
        height,
    }
}

pub fn vectorize(
    image: DynamicImage,
    options: &VectorizationOptions,
) -> Result<SvgFile, ConversionError> {
    if image.width() == 0 || image.height() == 0 {
        return Err(ConversionError::VectorizeFailed(
            "input image has no pixels".to_string(),
        ));
    }
    let config = build_vtracer_config(options)?;
    let color_image = dynamic_image_to_color_image(&image);
    vtracer::convert(color_image, config).map_err(ConversionError::VectorizeFailed)
}

pub fn vectorize_to_string(
    image: DynamicImage,
    options: &VectorizationOptions,
) -> Result<String, ConversionError> {
    Ok(vectorize(image, options)?.to_string())
}

pub fn vectorize_to_path(
    image: DynamicImage,
    path: &Path,
    options: &VectorizationOptions,
) -> Result<(), ConversionError> {
    output::validate_path(path)?;
    let svg = vectorize_to_string(image, options)?;
    output::write_file(path, svg.as_bytes())
}
