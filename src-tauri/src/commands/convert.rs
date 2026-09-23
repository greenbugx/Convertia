use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::conversion::{
    self, ConversionOptions, EncodeOptions, ImageFormat, PngCompression, TransformOptions,
    VectorColorMode, VectorPreset, VectorizationOptions,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImage {
    pub path: String,
    #[serde(default)]
    pub rotation: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertImagesRequest {
    pub images: Vec<SourceImage>,
    pub target_format: ImageFormat,
    pub jpeg_quality: Option<u8>,
    pub png_compression: Option<PngCompression>,
    pub webp_quality: Option<u8>,
    pub avif_quality: Option<u8>,
    pub avif_speed: Option<u8>,
    pub svg_preset: Option<VectorPreset>,
    pub svg_color_mode: Option<VectorColorMode>,
    pub svg_detail: Option<u8>,
    pub svg_smoothness: Option<u8>,
    pub svg_color_detail: Option<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConversionResult {
    pub source_name: String,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertImagesResponse {
    pub output_directory: String,
    pub results: Vec<ImageConversionResult>,
}

#[tauri::command]
pub async fn convert_images(
    app: AppHandle,
    request: ConvertImagesRequest,
) -> Result<ConvertImagesResponse, String> {
    tauri::async_runtime::spawn_blocking(move || run_conversion(&app, &request))
        .await
        .map_err(|e| format!("Conversion task failed to complete: {e}"))?
}

fn run_conversion(
    app: &AppHandle,
    request: &ConvertImagesRequest,
) -> Result<ConvertImagesResponse, String> {
    if request.images.is_empty() {
        return Err("No images were provided for conversion".to_string());
    }
    let output_directory = resolve_output_directory(app);
    fs::create_dir_all(&output_directory).map_err(|e| {
        format!(
            "Could not create output directory {}: {e}",
            output_directory.display()
        )
    })?;
    Ok(convert_batch(request, &output_directory))
}

pub fn convert_batch(
    request: &ConvertImagesRequest,
    output_directory: &Path,
) -> ConvertImagesResponse {
    let encode = EncodeOptions {
        jpeg_quality: request.jpeg_quality,
        png_compression: request.png_compression,
        webp_quality: request.webp_quality,
        avif_quality: request.avif_quality,
        avif_speed: request.avif_speed,
    };
    let defaults = VectorizationOptions::default();
    let vectorize = VectorizationOptions {
        preset: request.svg_preset.unwrap_or(defaults.preset),
        color_mode: request.svg_color_mode.unwrap_or(defaults.color_mode),
        detail: request.svg_detail.unwrap_or(defaults.detail),
        smoothness: request.svg_smoothness.unwrap_or(defaults.smoothness),
        color_detail: request.svg_color_detail.or(defaults.color_detail),
    };
    let mut results = Vec::with_capacity(request.images.len());
    for source in &request.images {
        results.push(convert_one(
            source,
            request.target_format,
            &encode,
            &vectorize,
            output_directory,
        ));
    }
    ConvertImagesResponse {
        output_directory: output_directory.to_string_lossy().into_owned(),
        results,
    }
}

fn convert_one(
    source: &SourceImage,
    format: ImageFormat,
    encode: &EncodeOptions,
    vectorize: &VectorizationOptions,
    output_directory: &Path,
) -> ImageConversionResult {
    let input_path = Path::new(&source.path);
    let display_name = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&source.path)
        .to_string();
    let stem = sanitize_stem(&display_name);
    let output_path = unique_output_path(output_directory, &stem, format.extension());
    let options = ConversionOptions {
        transform: TransformOptions {
            rotation: source.rotation,
            ..TransformOptions::default()
        },
        encode: encode.clone(),
        vectorize: *vectorize,
    };
    match conversion::convert_image(input_path, &output_path, format, &options) {
        Ok(converted) => ImageConversionResult {
            source_name: display_name,
            output_path: Some(converted.output_path.to_string_lossy().into_owned()),
            error: None,
        },
        Err(error) => ImageConversionResult {
            source_name: display_name,
            output_path: None,
            error: Some(error.to_string()),
        },
    }
}

fn resolve_output_directory(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .picture_dir()
        .ok()
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| {
            app.path()
                .download_dir()
                .ok()
                .filter(|p| !p.as_os_str().is_empty())
        })
        .unwrap_or_else(std::env::temp_dir);
    base.join("Convertia")
}

fn sanitize_stem(name: &str) -> String {
    let file_name = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "image".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unique_output_path(directory: &Path, stem: &str, extension: &str) -> PathBuf {
    let candidate = directory.join(format!("{stem}.{extension}"));
    if !candidate.exists() {
        return candidate;
    }
    for index in 1..1000 {
        let candidate = directory.join(format!("{stem} ({index}).{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{stem}-{}.{extension}", std::process::id()))
}
