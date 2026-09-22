use std::fs;
use std::path::{Path, PathBuf};

use image::{DynamicImage, RgbaImage};
use resvg::tiny_skia;
use resvg::usvg;

use super::error::ConversionError;

const MAX_RENDER_PIXELS: u64 = 67_108_864;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSize {
    pub width: u32,
    pub height: u32,
}

pub fn decode_path(
    path: &Path,
    preferred_size: Option<RenderSize>,
) -> Result<DynamicImage, ConversionError> {
    let data = fs::read(path).map_err(|error| {
        ConversionError::FileSystem(format!(
            "could not read SVG file {}: {error}",
            path.display()
        ))
    })?;
    let resources_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf);
    decode_with_resources(&data, resources_dir, preferred_size)
}

pub fn decode_bytes(
    data: &[u8],
    preferred_size: Option<RenderSize>,
) -> Result<DynamicImage, ConversionError> {
    decode_with_resources(data, None, preferred_size)
}

pub(crate) fn looks_like_svg(data: &[u8]) -> bool {
    let head_length = data.len().min(4096);
    let head = data[..head_length].to_ascii_lowercase();
    head.windows(4).any(|window| window == b"<svg")
}

fn decode_with_resources(
    data: &[u8],
    resources_dir: Option<PathBuf>,
    preferred_size: Option<RenderSize>,
) -> Result<DynamicImage, ConversionError> {
    if data.is_empty() {
        return Err(ConversionError::DecodeFailed(
            "SVG file is empty".to_string(),
        ));
    }
    let options = usvg::Options {
        resources_dir,
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(data, &options)
        .map_err(|error| ConversionError::DecodeFailed(format!("could not parse SVG: {error}")))?;
    render_tree(&tree, preferred_size)
}

fn render_tree(
    tree: &usvg::Tree,
    preferred_size: Option<RenderSize>,
) -> Result<DynamicImage, ConversionError> {
    let tree_size = tree.size();
    let (surface_width, surface_height, transform) = match preferred_size {
        Some(size) => (
            size.width,
            size.height,
            scale_transform(tree_size.width(), tree_size.height(), size),
        ),
        None => {
            let width = pixel_dimension(tree_size.width(), "width")?;
            let height = pixel_dimension(tree_size.height(), "height")?;
            (width, height, tiny_skia::Transform::identity())
        }
    };
    validate_surface(surface_width, surface_height)?;

    let mut pixmap = tiny_skia::Pixmap::new(surface_width, surface_height).ok_or_else(|| {
        ConversionError::DecodeFailed(format!(
            "could not allocate the {surface_width}x{surface_height} SVG render surface"
        ))
    })?;
    resvg::render(tree, transform, &mut pixmap.as_mut());
    let rgba = RgbaImage::from_raw(surface_width, surface_height, pixmap.take_demultiplied())
        .ok_or_else(|| {
            ConversionError::DecodeFailed(
                "SVG render surface does not match its dimensions".to_string(),
            )
        })?;
    Ok(DynamicImage::ImageRgba8(rgba))
}

fn scale_transform(tree_width: f32, tree_height: f32, size: RenderSize) -> tiny_skia::Transform {
    let scale_x = size.width as f32 / tree_width;
    let scale_y = size.height as f32 / tree_height;
    tiny_skia::Transform::from_scale(scale_x, scale_y)
}

fn pixel_dimension(value: f32, name: &str) -> Result<u32, ConversionError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(ConversionError::DecodeFailed(format!(
            "SVG has an invalid {name} dimension: {value}"
        )));
    }
    let rounded = value.ceil();
    if rounded >= u32::MAX as f32 {
        return Err(ConversionError::DecodeFailed(format!(
            "SVG {name} dimension is too large: {value}"
        )));
    }
    Ok(rounded as u32)
}

fn validate_surface(width: u32, height: u32) -> Result<(), ConversionError> {
    let pixels = u64::from(width) * u64::from(height);
    if pixels == 0 || pixels > MAX_RENDER_PIXELS {
        return Err(ConversionError::DecodeFailed(format!(
            "SVG render size {width}x{height} exceeds the supported limit of {MAX_RENDER_PIXELS} pixels"
        )));
    }
    Ok(())
}
