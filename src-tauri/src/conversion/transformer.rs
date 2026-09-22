use image::imageops::FilterType;
use image::DynamicImage;

use super::error::ConversionError;
use super::options::TransformOptions;

const MAX_RESIZE_PIXELS: u64 = 67_108_864;

pub fn apply(
    image: DynamicImage,
    options: &TransformOptions,
) -> Result<DynamicImage, ConversionError> {
    if options.rotation % 90 != 0 {
        return Err(ConversionError::InvalidTransform(format!(
            "rotation must be a multiple of 90 degrees, got {}",
            options.rotation
        )));
    }

    let mut image = match options.rotation % 360 {
        0 => image,
        90 => image.rotate90(),
        180 => image.rotate180(),
        _ => image.rotate270(),
    };

    if options.flip_horizontal {
        image = image.fliph();
    }
    if options.flip_vertical {
        image = image.flipv();
    }

    if let Some(crop) = &options.crop {
        if crop.width == 0 || crop.height == 0 {
            return Err(ConversionError::InvalidTransform(
                "crop width and height must be greater than zero".to_string(),
            ));
        }
        let right = u64::from(crop.x) + u64::from(crop.width);
        let bottom = u64::from(crop.y) + u64::from(crop.height);
        if right > u64::from(image.width()) || bottom > u64::from(image.height()) {
            return Err(ConversionError::InvalidTransform(format!(
                "crop region {}x{} at ({}, {}) exceeds image bounds {}x{}",
                crop.width,
                crop.height,
                crop.x,
                crop.y,
                image.width(),
                image.height()
            )));
        }
        image = image.crop_imm(crop.x, crop.y, crop.width, crop.height);
    }

    if let Some(resize) = &options.resize {
        if resize.width == 0 || resize.height == 0 {
            return Err(ConversionError::InvalidTransform(
                "resize width and height must be greater than zero".to_string(),
            ));
        }
        let pixels = u64::from(resize.width) * u64::from(resize.height);
        if pixels > MAX_RESIZE_PIXELS {
            return Err(ConversionError::InvalidTransform(format!(
                "resize target {}x{} is too large",
                resize.width, resize.height
            )));
        }
        image = if resize.exact {
            image.resize_exact(resize.width, resize.height, FilterType::Lanczos3)
        } else {
            image.resize(resize.width, resize.height, FilterType::Lanczos3)
        };
    }

    Ok(image)
}
