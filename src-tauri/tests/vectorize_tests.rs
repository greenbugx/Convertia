use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use convertia_lib::conversion::{
    convert_image, vectorizer_config, ConversionError, ConversionOptions, CropOptions, ImageFormat,
    ResizeOptions, TransformOptions, VectorColorMode, VectorPreset, VectorizationOptions,
};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};
use vtracer::ColorMode;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("convertia-vector-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("could not create test directory");
    dir
}

fn options(
    preset: VectorPreset,
    color_mode: VectorColorMode,
    detail: u8,
    smoothness: u8,
) -> VectorizationOptions {
    VectorizationOptions {
        preset,
        color_mode,
        detail,
        smoothness,
        color_detail: Some(60),
    }
}

fn logo_image(width: u32, height: u32) -> RgbImage {
    let mut image = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
    let radius = (width.min(height) / 3).max(2) as i32;
    let center_x = (width / 3) as i32;
    let center_y = (height / 3) as i32;
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let dx = x - center_x;
            let dy = y - center_y;
            if dx * dx + dy * dy <= radius * radius {
                image.put_pixel(x as u32, y as u32, Rgb([220, 40, 40]));
            }
        }
    }
    for y in (height / 2)..(height - 2) {
        for x in (width / 2)..(width - 2) {
            image.put_pixel(x, y, Rgb([30, 60, 200]));
        }
    }
    image
}

fn flat_illustration(width: u32, height: u32) -> RgbImage {
    let mut image = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let color = match (x * 3) / width {
                0 => Rgb([240, 200, 60]),
                1 => Rgb([60, 180, 120]),
                _ => Rgb([80, 90, 200]),
            };
            image.put_pixel(x, y, color);
        }
    }
    image
}

fn photograph_like(width: u32, height: u32) -> RgbImage {
    let mut state: u32 = 0x2468_ace1;
    let mut next = move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((state >> 16) & 0xff) as u8
    };
    ImageBuffer::from_fn(width, height, |x, y| {
        let base = ((x * 255 / width.max(1)) as u8 / 2) + ((y * 255 / height.max(1)) as u8 / 2);
        let noise = next() % 24;
        Rgb([
            base.saturating_add(noise),
            base.saturating_add(noise / 2),
            base,
        ])
    })
}

fn black_and_white_image(width: u32, height: u32) -> RgbImage {
    ImageBuffer::from_fn(width, height, |x, y| {
        let value = if ((x / 8) + (y / 8)) % 2 == 0 {
            20
        } else {
            235
        };
        Rgb([value, value, value])
    })
}

fn transparent_image(width: u32, height: u32) -> RgbaImage {
    ImageBuffer::from_fn(width, height, |x, y| {
        if x < width / 3 || y < height / 3 {
            Rgba([0, 0, 0, 0])
        } else {
            Rgba([250, 120, 30, 255])
        }
    })
}

fn write_fixture(
    dir: &Path,
    name: &str,
    image: &DynamicImage,
    format: image::ImageFormat,
) -> PathBuf {
    let path = dir.join(name);
    image
        .save_with_format(&path, format)
        .expect("could not write fixture");
    path
}

fn assert_vector_svg(path: &Path, expected_width: u32, expected_height: u32) -> String {
    let text = fs::read_to_string(path).expect("output should be readable text");
    assert!(!text.trim().is_empty(), "output should not be empty");
    assert!(
        text.starts_with("<?xml"),
        "output should start with an xml declaration"
    );
    assert!(
        text.contains("<svg"),
        "output should contain an svg root element"
    );
    assert!(
        text.contains("</svg>"),
        "output should close the svg root element"
    );
    assert!(text.contains("<path"), "output should contain vector paths");
    assert!(
        !text.contains("<image"),
        "output should not embed a raster image"
    );
    assert!(
        !text.contains("base64"),
        "output should not embed base64 data"
    );
    assert!(
        text.contains(&format!("width=\"{expected_width}\"")),
        "output should declare width {expected_width}"
    );
    assert!(
        text.contains(&format!("height=\"{expected_height}\"")),
        "output should declare height {expected_height}"
    );
    text
}

fn vectorize_fixture(
    dir: &Path,
    name: &str,
    image: DynamicImage,
    options: &VectorizationOptions,
) -> String {
    let input = write_fixture(dir, name, &image, image::ImageFormat::Png);
    let output = dir.join("out.svg");
    convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &conversion_options(options),
    )
    .expect("vectorization failed");
    assert_vector_svg(&output, image.width(), image.height())
}

fn conversion_options(vectorize: &VectorizationOptions) -> ConversionOptions {
    ConversionOptions {
        transform: TransformOptions::default(),
        encode: Default::default(),
        vectorize: *vectorize,
    }
}

#[test]
fn logo_preset_uses_its_own_baseline() {
    let config = vectorizer_config::build_logo_config();
    assert_eq!(config.filter_speckle, 6);
    assert_eq!(config.color_precision, 8);
    assert_eq!(config.layer_difference, 24);
    assert_eq!(config.corner_threshold, 80);
    assert_eq!(config.length_threshold, 3.0);
    assert_eq!(config.splice_threshold, 45);
    assert_eq!(config.max_iterations, 10);
    assert_eq!(config.path_precision, Some(2));
    assert!(matches!(config.color_mode, ColorMode::Color));
    assert!(matches!(
        config.hierarchical,
        vtracer::Hierarchical::Stacked
    ));
}

#[test]
fn presets_keep_their_documented_baselines_at_neutral_sliders() {
    let neutral_detail = 50;
    let neutral_smoothness = 50;
    let photo = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::Photo,
        VectorColorMode::Color,
        neutral_detail,
        neutral_smoothness,
    ))
    .expect("photo config should build");
    assert_eq!(photo.filter_speckle, 10);
    assert_eq!(photo.corner_threshold, 180);
    assert_eq!(photo.splice_threshold, 45);
    assert_eq!(photo.length_threshold, 4.0);

    let poster = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::Poster,
        VectorColorMode::Color,
        neutral_detail,
        neutral_smoothness,
    ))
    .expect("poster config should build");
    assert_eq!(poster.filter_speckle, 4);
    assert_eq!(poster.corner_threshold, 60);
    assert_eq!(poster.splice_threshold, 45);
    assert_eq!(poster.length_threshold, 4.0);

    let black_and_white = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::BlackAndWhite,
        VectorColorMode::BlackAndWhite,
        neutral_detail,
        neutral_smoothness,
    ))
    .expect("black and white config should build");
    assert_eq!(black_and_white.filter_speckle, 4);
    assert_eq!(black_and_white.corner_threshold, 60);
    assert!(matches!(black_and_white.color_mode, ColorMode::Binary));

    let logo = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::Logo,
        VectorColorMode::Color,
        neutral_detail,
        neutral_smoothness,
    ))
    .expect("logo config should build");
    assert_eq!(logo.filter_speckle, 6);
    assert_eq!(logo.corner_threshold, 80);
    assert_eq!(logo.length_threshold, 3.0);
}

#[test]
fn color_mode_overrides_the_preset_baseline() {
    let color_from_bw_preset = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::BlackAndWhite,
        VectorColorMode::Color,
        50,
        50,
    ))
    .expect("config should build");
    assert!(matches!(color_from_bw_preset.color_mode, ColorMode::Color));

    let binary_from_photo_preset = vectorizer_config::build_vtracer_config(&options(
        VectorPreset::Photo,
        VectorColorMode::BlackAndWhite,
        50,
        50,
    ))
    .expect("config should build");
    assert!(matches!(
        binary_from_photo_preset.color_mode,
        ColorMode::Binary
    ));
}

#[test]
fn detail_scales_regions_and_geometry_monotonically() {
    for preset in [
        VectorPreset::Logo,
        VectorPreset::Photo,
        VectorPreset::BlackAndWhite,
        VectorPreset::Poster,
    ] {
        let low = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            0,
            50,
        ))
        .expect("config should build");
        let middle = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            50,
            50,
        ))
        .expect("config should build");
        let high = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            100,
            50,
        ))
        .expect("config should build");

        assert!(low.filter_speckle >= middle.filter_speckle);
        assert!(middle.filter_speckle >= high.filter_speckle);
        assert!(low.layer_difference >= middle.layer_difference);
        assert!(middle.layer_difference >= high.layer_difference);
        assert!(low.length_threshold >= middle.length_threshold);
        assert!(middle.length_threshold >= high.length_threshold);
        assert!(low.filter_speckle > high.filter_speckle);
        assert!(low.length_threshold > high.length_threshold);
    }
}

#[test]
fn smoothness_scales_curvature_monotonically() {
    for preset in [
        VectorPreset::Logo,
        VectorPreset::Photo,
        VectorPreset::BlackAndWhite,
        VectorPreset::Poster,
    ] {
        let low = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            50,
            0,
        ))
        .expect("config should build");
        let middle = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            50,
            50,
        ))
        .expect("config should build");
        let high = vectorizer_config::build_vtracer_config(&options(
            preset,
            VectorColorMode::Color,
            50,
            100,
        ))
        .expect("config should build");

        assert!(low.corner_threshold <= middle.corner_threshold);
        assert!(middle.corner_threshold <= high.corner_threshold);
        assert!(low.splice_threshold <= middle.splice_threshold);
        assert!(middle.splice_threshold <= high.splice_threshold);
        assert!(low.length_threshold <= middle.length_threshold);
        assert!(middle.length_threshold <= high.length_threshold);
        assert!(low.splice_threshold < high.splice_threshold);
        assert!(low.length_threshold < high.length_threshold);
    }
}

#[test]
fn color_detail_scales_color_fidelity_monotonically() {
    for preset in [
        VectorPreset::Logo,
        VectorPreset::Photo,
        VectorPreset::Poster,
    ] {
        let mut low = options(preset, VectorColorMode::Color, 50, 50);
        low.color_detail = Some(0);
        let mut middle = options(preset, VectorColorMode::Color, 50, 50);
        middle.color_detail = Some(50);
        let mut high = options(preset, VectorColorMode::Color, 50, 50);
        high.color_detail = Some(100);

        let low_config =
            vectorizer_config::build_vtracer_config(&low).expect("config should build");
        let middle_config =
            vectorizer_config::build_vtracer_config(&middle).expect("config should build");
        let high_config =
            vectorizer_config::build_vtracer_config(&high).expect("config should build");

        assert!(low_config.color_precision <= middle_config.color_precision);
        assert!(middle_config.color_precision <= high_config.color_precision);
        assert!(low_config.layer_difference >= middle_config.layer_difference);
        assert!(middle_config.layer_difference >= high_config.layer_difference);
        assert!(low_config.color_precision < high_config.color_precision);
        assert!(low_config.layer_difference > high_config.layer_difference);
    }
}

#[test]
fn color_detail_is_ignored_in_black_and_white_mode() {
    let mut low = options(VectorPreset::Photo, VectorColorMode::BlackAndWhite, 50, 50);
    low.color_detail = Some(0);
    let mut high = options(VectorPreset::Photo, VectorColorMode::BlackAndWhite, 50, 50);
    high.color_detail = Some(100);
    let mut absent = options(VectorPreset::Photo, VectorColorMode::BlackAndWhite, 50, 50);
    absent.color_detail = None;

    let low_config = vectorizer_config::build_vtracer_config(&low).expect("config should build");
    let high_config = vectorizer_config::build_vtracer_config(&high).expect("config should build");
    let absent_config =
        vectorizer_config::build_vtracer_config(&absent).expect("config should build");

    assert_eq!(low_config.color_precision, high_config.color_precision);
    assert_eq!(low_config.layer_difference, high_config.layer_difference);
    assert_eq!(absent_config.color_precision, high_config.color_precision);
    assert_eq!(absent_config.layer_difference, high_config.layer_difference);
    assert_eq!(absent_config.filter_speckle, high_config.filter_speckle);
    assert_eq!(absent_config.corner_threshold, high_config.corner_threshold);
}

#[test]
fn out_of_range_sliders_are_rejected() {
    let mut detail = options(VectorPreset::Photo, VectorColorMode::Color, 101, 50);
    let result = vectorizer_config::build_vtracer_config(&detail);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));

    let mut smoothness = options(VectorPreset::Photo, VectorColorMode::Color, 50, 101);
    let result = vectorizer_config::build_vtracer_config(&smoothness);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));

    detail.detail = 50;
    detail.color_detail = Some(101);
    let result = vectorizer_config::build_vtracer_config(&detail);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));

    smoothness.smoothness = 50;
    let result = vectorizer_config::build_vtracer_config(&smoothness);
    assert!(result.is_ok());
}

#[test]
fn every_option_combination_produces_a_valid_config() {
    for preset in [
        VectorPreset::Logo,
        VectorPreset::Photo,
        VectorPreset::BlackAndWhite,
        VectorPreset::Poster,
    ] {
        for color_mode in [VectorColorMode::Color, VectorColorMode::BlackAndWhite] {
            for detail in [0u8, 50, 100] {
                for smoothness in [0u8, 50, 100] {
                    for color_detail in [None, Some(0), Some(50), Some(100)] {
                        let options = VectorizationOptions {
                            preset,
                            color_mode,
                            detail,
                            smoothness,
                            color_detail,
                        };
                        let config = vectorizer_config::build_vtracer_config(&options)
                            .expect("every valid combination should build");
                        vectorizer_config::validate_config(&config)
                            .expect("every built config should validate");
                    }
                }
            }
        }
    }
}

#[test]
fn png_logo_traces_to_vector_svg() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(64, 48));
    let output = dir.join("out.svg");
    let input = write_fixture(&dir, "logo.png", &image, image::ImageFormat::Png);
    convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &conversion_options(&VectorizationOptions::default()),
    )
    .expect("vectorization failed");
    assert_vector_svg(&output, 64, 48);
}

fn sample_dynamic(format: image::ImageFormat) -> DynamicImage {
    let rgb = logo_image(48, 32);
    if format == image::ImageFormat::Ico {
        DynamicImage::ImageRgba8(DynamicImage::ImageRgb8(rgb).to_rgba8())
    } else {
        DynamicImage::ImageRgb8(rgb)
    }
}

macro_rules! raster_to_svg_tests {
    ($( $name:ident: $input_name:literal, $input_format:expr );* $(;)?) => {
        $(
            #[test]
            fn $name() {
                let dir = temp_dir();
                let image = sample_dynamic($input_format);
                let input = write_fixture(&dir, $input_name, &image, $input_format);
                let output = dir.join(concat!(stringify!($name), ".svg"));
                convert_image(
                    &input,
                    &output,
                    ImageFormat::Svg,
                    &conversion_options(&VectorizationOptions::default()),
                )
                .expect("vectorization failed");
                assert_vector_svg(&output, 48, 32);
            }
        )*
    };
}

raster_to_svg_tests! {
    jpeg_to_svg: "input.jpg", image::ImageFormat::Jpeg;
    jpeg_extension_to_svg: "input.jpeg", image::ImageFormat::Jpeg;
    png_to_svg: "input.png", image::ImageFormat::Png;
    webp_to_svg: "input.webp", image::ImageFormat::WebP;
    avif_to_svg: "input.avif", image::ImageFormat::Avif;
    tiff_to_svg: "input.tiff", image::ImageFormat::Tiff;
    tif_extension_to_svg: "input.tif", image::ImageFormat::Tiff;
    bmp_to_svg: "input.bmp", image::ImageFormat::Bmp;
    ico_to_svg: "input.ico", image::ImageFormat::Ico;
}

#[test]
fn flat_illustration_traces() {
    let dir = temp_dir();
    vectorize_fixture(
        &dir,
        "illustration.png",
        DynamicImage::ImageRgb8(flat_illustration(96, 64)),
        &VectorizationOptions::default(),
    );
}

#[test]
fn photograph_like_source_traces() {
    let dir = temp_dir();
    vectorize_fixture(
        &dir,
        "photo.png",
        DynamicImage::ImageRgb8(photograph_like(96, 96)),
        &VectorizationOptions::default(),
    );
}

#[test]
fn black_and_white_source_traces() {
    let dir = temp_dir();
    vectorize_fixture(
        &dir,
        "bw.png",
        DynamicImage::ImageRgb8(black_and_white_image(64, 64)),
        &options(
            VectorPreset::BlackAndWhite,
            VectorColorMode::BlackAndWhite,
            50,
            50,
        ),
    );
}

#[test]
fn transparent_png_traces_without_a_background() {
    let dir = temp_dir();
    let text = vectorize_fixture(
        &dir,
        "transparent.png",
        DynamicImage::ImageRgba8(transparent_image(48, 48)),
        &VectorizationOptions::default(),
    );
    assert!(!text.contains("opacity=\"0\""));
}

#[test]
fn high_resolution_source_traces() {
    let dir = temp_dir();
    vectorize_fixture(
        &dir,
        "large.png",
        DynamicImage::ImageRgb8(flat_illustration(640, 480)),
        &VectorizationOptions::default(),
    );
}

#[test]
fn small_source_traces() {
    let dir = temp_dir();
    vectorize_fixture(
        &dir,
        "small.png",
        DynamicImage::ImageRgb8(logo_image(8, 8)),
        &VectorizationOptions::default(),
    );
}

#[test]
fn logo_photo_poster_and_bw_presets_all_trace() {
    for preset in [
        VectorPreset::Logo,
        VectorPreset::Photo,
        VectorPreset::BlackAndWhite,
        VectorPreset::Poster,
    ] {
        let dir = temp_dir();
        vectorize_fixture(
            &dir,
            "preset.png",
            DynamicImage::ImageRgb8(logo_image(40, 40)),
            &options(preset, VectorColorMode::Color, 60, 50),
        );
    }
}

#[test]
fn rotation_90_applies_through_the_common_pipeline() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(64, 32));
    let input = write_fixture(&dir, "rot.png", &image, image::ImageFormat::Png);
    let output = dir.join("out.svg");
    let options = ConversionOptions {
        transform: TransformOptions {
            rotation: 90,
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    convert_image(&input, &output, ImageFormat::Svg, &options).expect("vectorization failed");
    assert_vector_svg(&output, 32, 64);
}

#[test]
fn exact_resize_applies_through_the_common_pipeline() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(64, 48));
    let input = write_fixture(&dir, "resize.png", &image, image::ImageFormat::Png);
    let output = dir.join("out.svg");
    let options = ConversionOptions {
        transform: TransformOptions {
            resize: Some(ResizeOptions {
                width: 48,
                height: 24,
                exact: true,
            }),
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    convert_image(&input, &output, ImageFormat::Svg, &options).expect("vectorization failed");
    assert_vector_svg(&output, 48, 24);
}

#[test]
fn crop_applies_through_the_common_pipeline() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(64, 48));
    let input = write_fixture(&dir, "crop.png", &image, image::ImageFormat::Png);
    let output = dir.join("out.svg");
    let options = ConversionOptions {
        transform: TransformOptions {
            crop: Some(CropOptions {
                x: 8,
                y: 8,
                width: 32,
                height: 16,
            }),
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    convert_image(&input, &output, ImageFormat::Svg, &options).expect("vectorization failed");
    assert_vector_svg(&output, 32, 16);
}

#[test]
fn flips_apply_through_the_common_pipeline() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(48, 32));
    let input = write_fixture(&dir, "flip.png", &image, image::ImageFormat::Png);

    let plain_output = dir.join("plain.svg");
    convert_image(
        &input,
        &plain_output,
        ImageFormat::Svg,
        &ConversionOptions::default(),
    )
    .expect("vectorization failed");

    let flipped_output = dir.join("flipped.svg");
    let options = ConversionOptions {
        transform: TransformOptions {
            flip_horizontal: true,
            flip_vertical: true,
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    convert_image(&input, &flipped_output, ImageFormat::Svg, &options)
        .expect("vectorization failed");

    let plain = assert_vector_svg(&plain_output, 48, 32);
    let flipped = assert_vector_svg(&flipped_output, 48, 32);
    assert_ne!(plain, flipped);
}

#[test]
fn invalid_options_fail_the_conversion() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(16, 16));
    let input = write_fixture(&dir, "bad.png", &image, image::ImageFormat::Png);
    let output = dir.join("out.svg");
    let mut options = ConversionOptions::default();
    options.vectorize.detail = 101;
    let result = convert_image(&input, &output, ImageFormat::Svg, &options);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));
    assert!(!output.exists());
}

#[test]
fn missing_input_file_is_reported() {
    let dir = temp_dir();
    let input = dir.join("does-not-exist.png");
    let output = dir.join("out.svg");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &ConversionOptions::default(),
    );
    assert!(matches!(result, Err(ConversionError::InputNotFound(_))));
}

#[test]
fn output_path_failure_is_reported() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(16, 16));
    let input = write_fixture(&dir, "ok.png", &image, image::ImageFormat::Png);
    let output = dir.join("missing-dir").join("out.svg");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &ConversionOptions::default(),
    );
    assert!(matches!(result, Err(ConversionError::OutputPath(_))));
}

#[test]
fn source_file_is_not_modified() {
    let dir = temp_dir();
    let image = DynamicImage::ImageRgb8(logo_image(32, 32));
    let input = write_fixture(&dir, "original.png", &image, image::ImageFormat::Png);
    let before = fs::read(&input).expect("could not read input");
    let output = dir.join("out.svg");
    convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &ConversionOptions::default(),
    )
    .expect("vectorization failed");
    let after = fs::read(&input).expect("could not read input");
    assert_eq!(before, after);
}

#[test]
fn batch_conversion_writes_svg_files_through_the_command_layer() {
    use convertia_lib::commands::convert::{convert_batch, ConvertImagesRequest, SourceImage};

    let dir = temp_dir();
    let first = write_fixture(
        &dir,
        "first.png",
        &DynamicImage::ImageRgb8(logo_image(40, 32)),
        image::ImageFormat::Png,
    );
    let second = write_fixture(
        &dir,
        "second.jpg",
        &DynamicImage::ImageRgb8(logo_image(40, 32)),
        image::ImageFormat::Jpeg,
    );
    let output_dir = dir.join("out");
    fs::create_dir_all(&output_dir).expect("could not create output directory");

    let request = ConvertImagesRequest {
        images: vec![
            SourceImage {
                path: first.to_string_lossy().into_owned(),
                rotation: 0,
            },
            SourceImage {
                path: second.to_string_lossy().into_owned(),
                rotation: 90,
            },
        ],
        target_format: ImageFormat::Svg,
        jpeg_quality: None,
        png_compression: None,
        webp_quality: None,
        avif_quality: None,
        avif_speed: None,
        svg_preset: Some(VectorPreset::Logo),
        svg_color_mode: Some(VectorColorMode::Color),
        svg_detail: Some(60),
        svg_smoothness: Some(50),
        svg_color_detail: Some(60),
    };

    let response = convert_batch(&request, &output_dir);
    assert_eq!(response.results.len(), 2);
    assert!(response.results[0].error.is_none());
    assert!(response.results[1].error.is_none());

    let first_path = response.results[0]
        .output_path
        .as_ref()
        .expect("missing output path");
    assert!(first_path.ends_with("first.svg"));
    assert_vector_svg(Path::new(first_path), 40, 32);

    let second_path = response.results[1]
        .output_path
        .as_ref()
        .expect("missing output path");
    assert!(second_path.ends_with("second.svg"));
    assert_vector_svg(Path::new(second_path), 32, 40);
}
