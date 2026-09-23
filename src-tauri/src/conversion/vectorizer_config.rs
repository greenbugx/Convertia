use serde::{Deserialize, Serialize};
use vtracer::{ColorMode, Config, Preset};

use super::error::ConversionError;

const MAX_SPECKLE: usize = 64;
const MIN_SPECKLE: usize = 0;
const MIN_COLOR_PRECISION: i32 = 1;
const MAX_COLOR_PRECISION: i32 = 8;
const MIN_LAYER_DIFFERENCE: i32 = 1;
const MAX_LAYER_DIFFERENCE: i32 = 255;
const MIN_THRESHOLD: i32 = 1;
const MAX_THRESHOLD: i32 = 180;
const MIN_LENGTH_THRESHOLD: f64 = 0.5;
const MAX_LENGTH_THRESHOLD: f64 = 64.0;
const MAX_ITERATIONS: usize = 100;
const MAX_PATH_PRECISION: u32 = 10;

const DEFAULT_DETAIL: u8 = 60;
const DEFAULT_SMOOTHNESS: u8 = 50;
const DEFAULT_COLOR_DETAIL: u8 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum VectorPreset {
    #[serde(rename = "Logo")]
    Logo,
    #[serde(rename = "Photo")]
    #[default]
    Photo,
    #[serde(rename = "Black and white")]
    BlackAndWhite,
    #[serde(rename = "Poster")]
    Poster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum VectorColorMode {
    #[serde(rename = "Color")]
    #[default]
    Color,
    #[serde(rename = "Black and white")]
    BlackAndWhite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorizationOptions {
    pub preset: VectorPreset,
    pub color_mode: VectorColorMode,
    pub detail: u8,
    pub smoothness: u8,
    pub color_detail: Option<u8>,
}

impl Default for VectorizationOptions {
    fn default() -> Self {
        Self {
            preset: VectorPreset::default(),
            color_mode: VectorColorMode::default(),
            detail: DEFAULT_DETAIL,
            smoothness: DEFAULT_SMOOTHNESS,
            color_detail: Some(DEFAULT_COLOR_DETAIL),
        }
    }
}

pub fn build_logo_config() -> Config {
    let mut config = Config::from_preset(Preset::Poster);
    config.filter_speckle = 6;
    config.layer_difference = 24;
    config.corner_threshold = 80;
    config.length_threshold = 3.0;
    config
}

pub fn build_vtracer_config(options: &VectorizationOptions) -> Result<Config, ConversionError> {
    validate_options(options)?;
    let mut config = match options.preset {
        VectorPreset::Logo => build_logo_config(),
        VectorPreset::Photo => Config::from_preset(Preset::Photo),
        VectorPreset::BlackAndWhite => Config::from_preset(Preset::Bw),
        VectorPreset::Poster => Config::from_preset(Preset::Poster),
    };
    apply_color_mode(&mut config, options.color_mode);
    apply_detail(&mut config, options.detail);
    apply_smoothness(&mut config, options.smoothness);
    if matches!(options.color_mode, VectorColorMode::Color) {
        if let Some(color_detail) = options.color_detail {
            apply_color_detail(&mut config, color_detail);
        }
    }
    validate_config(&config)?;
    Ok(config)
}

pub fn apply_color_mode(config: &mut Config, color_mode: VectorColorMode) {
    config.color_mode = match color_mode {
        VectorColorMode::Color => ColorMode::Color,
        VectorColorMode::BlackAndWhite => ColorMode::Binary,
    };
}

pub fn apply_detail(config: &mut Config, detail: u8) {
    let t = f64::from(detail) / 100.0;
    let region_factor = 1.5 - t;
    config.filter_speckle = clamp_usize(
        (config.filter_speckle as f64 * region_factor).round(),
        MIN_SPECKLE,
        MAX_SPECKLE,
    );
    config.length_threshold = clamp_f64(
        config.length_threshold * region_factor,
        MIN_LENGTH_THRESHOLD,
        MAX_LENGTH_THRESHOLD,
    );
    config.layer_difference = clamp_i32(
        (config.layer_difference as f64 * region_factor).round(),
        MIN_LAYER_DIFFERENCE,
        MAX_LAYER_DIFFERENCE,
    );
}

pub fn apply_smoothness(config: &mut Config, smoothness: u8) {
    let t = f64::from(smoothness) / 100.0;
    let corner_factor = 0.5 + t;
    config.corner_threshold = clamp_i32(
        (config.corner_threshold as f64 * corner_factor).round(),
        MIN_THRESHOLD,
        MAX_THRESHOLD,
    );
    let length_factor = 0.6 + 0.8 * t;
    config.length_threshold = clamp_f64(
        config.length_threshold * length_factor,
        MIN_LENGTH_THRESHOLD,
        MAX_LENGTH_THRESHOLD,
    );
    let splice_factor = 0.5 + t;
    config.splice_threshold = clamp_i32(
        (config.splice_threshold as f64 * splice_factor).round(),
        MIN_THRESHOLD,
        MAX_THRESHOLD,
    );
}

pub fn apply_color_detail(config: &mut Config, color_detail: u8) {
    let t = f64::from(color_detail) / 100.0;
    let precision_floor = (config.color_precision as f64 * 0.5).max(f64::from(MIN_COLOR_PRECISION));
    config.color_precision = clamp_i32(
        lerp(precision_floor, f64::from(MAX_COLOR_PRECISION), t),
        MIN_COLOR_PRECISION,
        MAX_COLOR_PRECISION,
    );
    let layer_factor = 1.4 - 0.8 * t;
    config.layer_difference = clamp_i32(
        (config.layer_difference as f64 * layer_factor).round(),
        MIN_LAYER_DIFFERENCE,
        MAX_LAYER_DIFFERENCE,
    );
}

pub fn validate_options(options: &VectorizationOptions) -> Result<(), ConversionError> {
    validate_percent("Detail", options.detail)?;
    validate_percent("Smoothness", options.smoothness)?;
    if let Some(color_detail) = options.color_detail {
        validate_percent("Color detail", color_detail)?;
    }
    Ok(())
}

pub fn validate_config(config: &Config) -> Result<(), ConversionError> {
    if config.filter_speckle > MAX_SPECKLE {
        return Err(invalid(format!(
            "speckle filter must be between {MIN_SPECKLE} and {MAX_SPECKLE}, got {}",
            config.filter_speckle
        )));
    }
    if !(MIN_COLOR_PRECISION..=MAX_COLOR_PRECISION).contains(&config.color_precision) {
        return Err(invalid(format!(
            "color precision must be between {MIN_COLOR_PRECISION} and {MAX_COLOR_PRECISION}, got {}",
            config.color_precision
        )));
    }
    if !(MIN_LAYER_DIFFERENCE..=MAX_LAYER_DIFFERENCE).contains(&config.layer_difference) {
        return Err(invalid(format!(
            "layer difference must be between {MIN_LAYER_DIFFERENCE} and {MAX_LAYER_DIFFERENCE}, got {}",
            config.layer_difference
        )));
    }
    if !(MIN_THRESHOLD..=MAX_THRESHOLD).contains(&config.corner_threshold) {
        return Err(invalid(format!(
            "corner threshold must be between {MIN_THRESHOLD} and {MAX_THRESHOLD}, got {}",
            config.corner_threshold
        )));
    }
    if !(MIN_THRESHOLD..=MAX_THRESHOLD).contains(&config.splice_threshold) {
        return Err(invalid(format!(
            "splice threshold must be between {MIN_THRESHOLD} and {MAX_THRESHOLD}, got {}",
            config.splice_threshold
        )));
    }
    if !config.length_threshold.is_finite()
        || config.length_threshold < MIN_LENGTH_THRESHOLD
        || config.length_threshold > MAX_LENGTH_THRESHOLD
    {
        return Err(invalid(format!(
            "segment length threshold must be between {MIN_LENGTH_THRESHOLD} and {MAX_LENGTH_THRESHOLD}, got {}",
            config.length_threshold
        )));
    }
    if config.max_iterations == 0 || config.max_iterations > MAX_ITERATIONS {
        return Err(invalid(format!(
            "max iterations must be between 1 and {MAX_ITERATIONS}, got {}",
            config.max_iterations
        )));
    }
    if let Some(path_precision) = config.path_precision {
        if path_precision > MAX_PATH_PRECISION {
            return Err(invalid(format!(
                "path precision must be at most {MAX_PATH_PRECISION}, got {path_precision}"
            )));
        }
    }
    Ok(())
}

fn validate_percent(name: &str, value: u8) -> Result<(), ConversionError> {
    if value > 100 {
        return Err(invalid(format!(
            "{name} must be between 0 and 100, got {value}"
        )));
    }
    Ok(())
}

fn invalid(detail: String) -> ConversionError {
    ConversionError::InvalidOptions(detail)
}

fn lerp(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}

fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn clamp_i32(value: f64, min: i32, max: i32) -> i32 {
    (value.round() as i32).clamp(min, max)
}

fn clamp_usize(value: f64, min: usize, max: usize) -> usize {
    let rounded = if value.is_finite() && value > 0.0 {
        value.round() as usize
    } else {
        0
    };
    rounded.clamp(min, max)
}
