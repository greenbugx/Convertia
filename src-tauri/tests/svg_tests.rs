use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use convertia_lib::conversion::{
    convert_bytes, convert_image, ConversionError, ConversionOptions, CropOptions, ImageFormat,
    InputFormat, ResizeOptions, TransformOptions,
};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgb, RgbImage};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("convertia-svg-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("could not create test directory");
    dir
}

fn write_svg(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).expect("could not write svg fixture");
    path
}

const SVG_RED_RECT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect x="0" y="0" width="64" height="32" fill="#ff0000"/></svg>"##;

const SVG_HALVES: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect x="0" y="0" width="16" height="32" fill="#ff0000"/><rect x="16" y="0" width="16" height="32" fill="#00ff00"/></svg>"##;

const SVG_STACKED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect x="0" y="0" width="32" height="16" fill="#ff0000"/><rect x="0" y="16" width="32" height="16" fill="#0000ff"/></svg>"##;

const SVG_SEMI_TRANSPARENT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect x="0" y="0" width="16" height="32" fill="#ff0000" fill-opacity="0.5"/></svg>"##;

fn convert_and_reopen(
    input: &Path,
    output: &Path,
    format: ImageFormat,
    options: &ConversionOptions,
) -> (image::ImageFormat, DynamicImage) {
    convert_image(input, output, format, options).expect("conversion failed");
    let reader = ImageReader::open(output)
        .expect("could not open output")
        .with_guessed_format()
        .expect("could not guess output format");
    let detected = reader.format().expect("output format missing");
    let decoded = reader.decode().expect("could not decode output");
    (detected, decoded)
}

fn svg_conversion(
    dir: &Path,
    contents: &str,
    output_name: &str,
    format: ImageFormat,
) -> (image::ImageFormat, DynamicImage) {
    let input = write_svg(dir, "input.svg", contents);
    let output = dir.join(output_name);
    convert_and_reopen(&input, &output, format, &ConversionOptions::default())
}

macro_rules! svg_output_tests {
    ($( $name:ident: $output_name:literal, $output_format:expr, $detected:expr );* $(;)?) => {
        $(
            #[test]
            fn $name() {
                let dir = temp_dir();
                let (format, image) =
                    svg_conversion(&dir, SVG_RED_RECT, $output_name, $output_format);
                assert_eq!(format, $detected);
                assert_eq!(image.dimensions(), (64, 32));
            }
        )*
    };
}

svg_output_tests! {
    svg_to_png: "out.png", ImageFormat::Png, image::ImageFormat::Png;
    svg_to_jpeg: "out.jpg", ImageFormat::Jpeg, image::ImageFormat::Jpeg;
    svg_to_webp: "out.webp", ImageFormat::Webp, image::ImageFormat::WebP;
    svg_to_avif: "out.avif", ImageFormat::Avif, image::ImageFormat::Avif;
    svg_to_tiff: "out.tiff", ImageFormat::Tiff, image::ImageFormat::Tiff;
    svg_to_ico: "out.ico", ImageFormat::Ico, image::ImageFormat::Ico;
    svg_to_bmp: "out.bmp", ImageFormat::Bmp, image::ImageFormat::Bmp;
}

#[test]
fn svg_dimensions_come_from_width_and_height() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_RED_RECT, "out.png", ImageFormat::Png);
    assert_eq!(image.dimensions(), (64, 32));
    let rgb = image.to_rgb8();
    assert_eq!(rgb.get_pixel(4, 4).0, [255, 0, 0]);
}

#[test]
fn svg_dimensions_come_from_viewbox_when_size_is_missing() {
    let dir = temp_dir();
    let contents = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 32"><rect x="0" y="0" width="64" height="32" fill="#00ff00"/></svg>"##;
    let (_, image) = svg_conversion(&dir, contents, "out.png", ImageFormat::Png);
    assert_eq!(image.dimensions(), (64, 32));
}

#[test]
fn svg_transparency_is_preserved_for_png() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_SEMI_TRANSPARENT, "out.png", ImageFormat::Png);
    let rgba = image.to_rgba8();
    let transparent = rgba.get_pixel(24, 16).0;
    assert_eq!(transparent[3], 0);
    let semi = rgba.get_pixel(8, 16).0;
    assert!(semi[3] > 100 && semi[3] < 155);
    assert!(semi[0] > 100);
}

#[test]
fn svg_jpeg_output_flattens_transparency_on_white() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_SEMI_TRANSPARENT, "out.jpg", ImageFormat::Jpeg);
    assert_eq!(image.dimensions(), (32, 32));
    let rgb = image.to_rgb8();
    let flattened = rgb.get_pixel(24, 16).0;
    assert!(flattened[0] > 240 && flattened[1] > 240 && flattened[2] > 240);
}

#[test]
fn svg_webp_output_preserves_alpha() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_SEMI_TRANSPARENT, "out.webp", ImageFormat::Webp);
    let rgba = image.to_rgba8();
    let semi = rgba.get_pixel(8, 16).0;
    assert!(semi[3] > 0 && semi[3] < 255);
}

#[test]
fn svg_avif_output_preserves_alpha() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_SEMI_TRANSPARENT, "out.avif", ImageFormat::Avif);
    let rgba = image.to_rgba8();
    let semi = rgba.get_pixel(8, 16).0;
    assert!(semi[3] > 0 && semi[3] < 255);
}

#[test]
fn svg_tiff_output_preserves_alpha() {
    let dir = temp_dir();
    let (_, image) = svg_conversion(&dir, SVG_SEMI_TRANSPARENT, "out.tiff", ImageFormat::Tiff);
    let rgba = image.to_rgba8();
    let semi = rgba.get_pixel(8, 16).0;
    assert!(semi[3] > 0 && semi[3] < 255);
}

#[test]
fn svg_with_relative_image_resource_renders_the_resource() {
    let dir = temp_dir();
    let source_dir = dir.join("source");
    fs::create_dir_all(&source_dir).expect("could not create source directory");
    let solid_red: RgbImage = ImageBuffer::from_fn(8, 8, |_, _| Rgb([255, 0, 0]));
    DynamicImage::ImageRgb8(solid_red)
        .save_with_format(source_dir.join("asset.png"), image::ImageFormat::Png)
        .expect("could not write resource fixture");
    let contents = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><image href="asset.png" x="0" y="0" width="32" height="32"/></svg>"##;
    let input = write_svg(&source_dir, "input.svg", contents);
    let output = dir.join("out.png");

    let (_, image) = convert_and_reopen(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );

    assert_eq!(image.dimensions(), (32, 32));
    let rgba = image.to_rgba8();
    let rendered = rgba.get_pixel(16, 16).0;
    assert!(rendered[0] > 200 && rendered[1] < 60 && rendered[3] == 255);
}

#[test]
fn svg_with_text_converts() {
    let dir = temp_dir();
    let contents = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><text x="4" y="22" font-family="sans-serif" font-size="14" fill="#000000">Hello</text></svg>"##;
    let (_, image) = svg_conversion(&dir, contents, "out.png", ImageFormat::Png);
    assert_eq!(image.dimensions(), (64, 32));
}

#[test]
fn svg_rotation_uses_the_common_transform_pipeline() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_RED_RECT);
    let output = dir.join("out.png");
    let options = ConversionOptions {
        transform: TransformOptions {
            rotation: 90,
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (32, 64));
}

#[test]
fn svg_resize_uses_the_common_transform_pipeline() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_RED_RECT);
    let output = dir.join("out.png");
    let options = ConversionOptions {
        transform: TransformOptions {
            resize: Some(ResizeOptions {
                width: 16,
                height: 8,
                exact: true,
            }),
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (16, 8));
}

#[test]
fn svg_crop_uses_the_common_transform_pipeline() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_RED_RECT);
    let output = dir.join("out.png");
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
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (32, 16));
    let rgb = image.to_rgb8();
    assert_eq!(rgb.get_pixel(4, 4).0, [255, 0, 0]);
}

#[test]
fn svg_horizontal_flip_uses_the_common_transform_pipeline() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_HALVES);
    let output = dir.join("out.png");
    let options = ConversionOptions {
        transform: TransformOptions {
            flip_horizontal: true,
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    let rgb = image.to_rgb8();
    assert_eq!(rgb.get_pixel(4, 16).0, [0, 255, 0]);
    assert_eq!(rgb.get_pixel(28, 16).0, [255, 0, 0]);
}

#[test]
fn svg_vertical_flip_uses_the_common_transform_pipeline() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_STACKED);
    let output = dir.join("out.png");
    let options = ConversionOptions {
        transform: TransformOptions {
            flip_vertical: true,
            ..TransformOptions::default()
        },
        ..ConversionOptions::default()
    };
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    let rgb = image.to_rgb8();
    assert_eq!(rgb.get_pixel(16, 4).0, [0, 0, 255]);
    assert_eq!(rgb.get_pixel(16, 28).0, [255, 0, 0]);
}

#[test]
fn svg_is_decoded_from_bytes_through_the_common_entry_point() {
    let dir = temp_dir();
    let output = dir.join("out.png");
    let converted = convert_bytes(
        SVG_RED_RECT.as_bytes(),
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    )
    .expect("conversion failed");
    assert_eq!((converted.width, converted.height), (64, 32));
}

#[test]
fn malformed_svg_is_rejected() {
    let dir = temp_dir();
    let input = write_svg(
        &dir,
        "input.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"",
    );
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn empty_svg_file_is_rejected() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", "");
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn invalid_svg_dimensions_are_rejected() {
    let dir = temp_dir();
    let contents = r##"<svg xmlns="http://www.w3.org/2000/svg" width="0" height="0"></svg>"##;
    let input = write_svg(&dir, "input.svg", contents);
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn oversized_svg_render_is_rejected() {
    let dir = temp_dir();
    let contents =
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="20000" height="20000"></svg>"##;
    let input = write_svg(&dir, "input.svg", contents);
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn missing_svg_input_file_is_reported() {
    let dir = temp_dir();
    let input = dir.join("does-not-exist.svg");
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn svg_output_path_failure_is_reported() {
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_RED_RECT);
    let output = dir.join("missing-dir").join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_err());
}

#[test]
fn svg_input_to_svg_output_stays_unsupported() {
    assert_eq!(ImageFormat::from_extension("svg"), Some(ImageFormat::Svg));
    let dir = temp_dir();
    let input = write_svg(&dir, "input.svg", SVG_RED_RECT);
    assert_eq!(InputFormat::from_path(&input), Some(InputFormat::Svg));
    let output = dir.join("out.svg");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Svg,
        &ConversionOptions::default(),
    );
    assert!(matches!(
        result,
        Err(ConversionError::UnsupportedOutputFormat(_))
    ));
    assert!(!output.exists());

    let raster_output = dir.join("out.png");
    let result = convert_image(
        &input,
        &raster_output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(result.is_ok());
}
