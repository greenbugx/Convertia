use std::borrow::Cow;
use std::io::{Cursor, Seek, Write};
use std::path::Path;

use image::codecs::avif::AvifEncoder;
use image::codecs::bmp::BmpEncoder;
use image::codecs::ico::IcoEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::tiff::TiffEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, RgbImage};

use super::error::ConversionError;
use super::formats::ImageFormat;
use super::options::{EncodeOptions, PngCompression};
use super::output;

const DEFAULT_JPEG_QUALITY: u8 = 85;
const DEFAULT_AVIF_QUALITY: u8 = 85;
const DEFAULT_AVIF_SPEED: u8 = 6;
const MAX_ICO_DIMENSION: u32 = 256;

pub fn encode_to_path(
    image: &DynamicImage,
    path: &Path,
    format: ImageFormat,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    output::validate_path(path)?;
    let file_data = encode_to_vec(image, format, options)?;
    output::write_file(path, &file_data)
}

pub fn encode_to_vec(
    image: &DynamicImage,
    format: ImageFormat,
    options: &EncodeOptions,
) -> Result<Vec<u8>, ConversionError> {
    let mut buffer = Cursor::new(Vec::new());
    encode_to_writer(image, &mut buffer, format, options)?;
    Ok(buffer.into_inner())
}

fn encode_to_writer<W: Write + Seek>(
    image: &DynamicImage,
    mut writer: W,
    format: ImageFormat,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    match format {
        ImageFormat::Jpeg => encode_jpeg(image, &mut writer, options),
        ImageFormat::Png => encode_png(image, &mut writer, options),
        ImageFormat::Webp => encode_webp(image, &mut writer, options),
        ImageFormat::Avif => encode_avif(image, &mut writer, options),
        ImageFormat::Tiff => image
            .write_with_encoder(TiffEncoder::new(&mut writer))
            .map_err(encode_failed),
        ImageFormat::Ico => encode_ico(image, &mut writer),
        ImageFormat::Bmp => encode_bmp(image, &mut writer),
        ImageFormat::Svg => Err(ConversionError::UnsupportedOutputFormat(
            "SVG output is produced by the vectorizer, not the raster encoder".to_string(),
        )),
    }
}

fn encode_failed(error: image::ImageError) -> ConversionError {
    ConversionError::EncodeFailed(error.to_string())
}

fn encode_jpeg<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    let quality = options.jpeg_quality.unwrap_or(DEFAULT_JPEG_QUALITY);
    if !(1..=100).contains(&quality) {
        return Err(ConversionError::InvalidOptions(format!(
            "JPEG quality must be between 1 and 100, got {quality}"
        )));
    }
    let rgb = flatten_on_white(image);
    JpegEncoder::new_with_quality(writer, quality)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(encode_failed)
}

fn encode_png<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    let compression = match options.png_compression.unwrap_or_default() {
        PngCompression::Fast => CompressionType::Fast,
        PngCompression::Balanced => CompressionType::Default,
        PngCompression::Maximum => CompressionType::Best,
    };
    let encoder = PngEncoder::new_with_quality(writer, compression, FilterType::Adaptive);
    image.write_with_encoder(encoder).map_err(encode_failed)
}

fn encode_webp<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    match options.webp_quality {
        Some(quality) => {
            if !(1..=100).contains(&quality) {
                return Err(ConversionError::InvalidOptions(format!(
                    "WebP quality must be between 1 and 100, got {quality}"
                )));
            }
            encode_webp_lossy(image, writer, quality)
        }
        None => {
            let image = ensure_8bit(image);
            image
                .write_with_encoder(WebPEncoder::new_lossless(writer))
                .map_err(encode_failed)
        }
    }
}

fn encode_webp_lossy<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
    quality: u8,
) -> Result<(), ConversionError> {
    let image = ensure_8bit(image);
    let encoded = if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
            .encode_simple(false, f32::from(quality))
    } else {
        let rgb = image.to_rgb8();
        webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height())
            .encode_simple(false, f32::from(quality))
    };
    let encoded = encoded
        .map_err(|e| ConversionError::EncodeFailed(format!("lossy WebP encoding failed: {e:?}")))?;
    writer
        .write_all(&encoded)
        .map_err(|e| ConversionError::EncodeFailed(format!("could not write WebP data: {e}")))
}

fn encode_avif<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
    options: &EncodeOptions,
) -> Result<(), ConversionError> {
    let quality = options.avif_quality.unwrap_or(DEFAULT_AVIF_QUALITY);
    if !(1..=100).contains(&quality) {
        return Err(ConversionError::InvalidOptions(format!(
            "AVIF quality must be between 1 and 100, got {quality}"
        )));
    }
    let speed = options.avif_speed.unwrap_or(DEFAULT_AVIF_SPEED);
    if !(1..=10).contains(&speed) {
        return Err(ConversionError::InvalidOptions(format!(
            "AVIF speed must be between 1 and 10, got {speed}"
        )));
    }
    let image = ensure_integer_samples(image);
    let encoder = AvifEncoder::new_with_speed_quality(writer, speed, quality);
    image.write_with_encoder(encoder).map_err(encode_failed)
}

fn encode_ico<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
) -> Result<(), ConversionError> {
    if image.width() > MAX_ICO_DIMENSION || image.height() > MAX_ICO_DIMENSION {
        return Err(ConversionError::InvalidOptions(format!(
            "ICO output supports at most 256x256 pixels, got {}x{}",
            image.width(),
            image.height()
        )));
    }
    let image = match image {
        DynamicImage::ImageRgba8(_) => Cow::Borrowed(image),
        _ => Cow::Owned(DynamicImage::ImageRgba8(image.to_rgba8())),
    };
    image
        .write_with_encoder(IcoEncoder::new(writer))
        .map_err(encode_failed)
}

fn encode_bmp<W: Write + Seek>(
    image: &DynamicImage,
    writer: &mut W,
) -> Result<(), ConversionError> {
    let image = ensure_8bit(image);
    image
        .write_with_encoder(BmpEncoder::new(writer))
        .map_err(encode_failed)
}

fn ensure_8bit(image: &DynamicImage) -> Cow<'_, DynamicImage> {
    match image {
        DynamicImage::ImageLuma8(_)
        | DynamicImage::ImageLumaA8(_)
        | DynamicImage::ImageRgb8(_)
        | DynamicImage::ImageRgba8(_) => Cow::Borrowed(image),
        _ => {
            if image.color().has_alpha() {
                Cow::Owned(DynamicImage::ImageRgba8(image.to_rgba8()))
            } else {
                Cow::Owned(DynamicImage::ImageRgb8(image.to_rgb8()))
            }
        }
    }
}

fn ensure_integer_samples(image: &DynamicImage) -> Cow<'_, DynamicImage> {
    match image {
        DynamicImage::ImageRgb32F(_) => Cow::Owned(DynamicImage::ImageRgb8(image.to_rgb8())),
        DynamicImage::ImageRgba32F(_) => Cow::Owned(DynamicImage::ImageRgba8(image.to_rgba8())),
        _ => Cow::Borrowed(image),
    }
}

fn flatten_on_white(image: &DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.to_rgb8();
    }
    let rgba = image.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (target, source) in rgb.pixels_mut().zip(rgba.pixels()) {
        let alpha = u32::from(source[3]);
        let inverse = 255 - alpha;
        for channel in 0..3 {
            target.0[channel] =
                ((u32::from(source[channel]) * alpha + 255 * inverse + 127) / 255) as u8;
        }
    }
    rgb
}
