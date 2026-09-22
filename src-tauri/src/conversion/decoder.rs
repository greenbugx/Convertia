use std::io::{BufRead, BufReader, Cursor, Seek};
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageReader};

use super::error::ConversionError;
use super::formats::ImageFormat;

pub fn decode_path(path: &Path) -> Result<(DynamicImage, ImageFormat), ConversionError> {
    if path.as_os_str().is_empty() {
        return Err(ConversionError::InvalidInputPath(
            "input path is empty".to_string(),
        ));
    }
    if !path.exists() {
        return Err(ConversionError::InputNotFound(path.display().to_string()));
    }
    if !path.is_file() {
        return Err(ConversionError::InvalidInputPath(format!(
            "input path is not a file: {}",
            path.display()
        )));
    }
    let reader = ImageReader::open(path)
        .map_err(|e| {
            ConversionError::FileSystem(format!("could not open {}: {e}", path.display()))
        })?
        .with_guessed_format()
        .map_err(|e| {
            ConversionError::FileSystem(format!("could not read {}: {e}", path.display()))
        })?;
    decode_reader(reader)
}

pub fn decode_bytes(data: &[u8]) -> Result<(DynamicImage, ImageFormat), ConversionError> {
    if data.is_empty() {
        return Err(ConversionError::DecodeFailed(
            "input file is empty".to_string(),
        ));
    }
    let reader = ImageReader::new(BufReader::new(Cursor::new(data)))
        .with_guessed_format()
        .map_err(|e| ConversionError::FileSystem(format!("could not read input data: {e}")))?;
    decode_reader(reader)
}

fn decode_reader<R: BufRead + Seek>(
    reader: ImageReader<R>,
) -> Result<(DynamicImage, ImageFormat), ConversionError> {
    let raster_format = reader.format().ok_or_else(|| {
        ConversionError::UnsupportedInputFormat(
            "could not determine the image format from the file content".to_string(),
        )
    })?;
    let format = ImageFormat::from_raster(raster_format).ok_or_else(|| {
        ConversionError::UnsupportedInputFormat(format!("{raster_format:?} is not supported"))
    })?;
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| ConversionError::DecodeFailed(e.to_string()))?;
    let orientation = decoder
        .orientation()
        .map_err(|e| ConversionError::DecodeFailed(format!("could not read orientation: {e}")))?;
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|e| ConversionError::DecodeFailed(e.to_string()))?;
    image.apply_orientation(orientation);
    Ok((image, format))
}
