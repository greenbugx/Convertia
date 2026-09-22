use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use convertia_lib::conversion::{
    convert_bytes, convert_image, ConversionError, ConversionOptions, CropOptions, EncodeOptions,
    ImageFormat, PngCompression, ResizeOptions, TransformOptions,
};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgb, RgbImage};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("convertia-tests-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("could not create test directory");
    dir
}

fn gradient_image(width: u32, height: u32) -> RgbImage {
    ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([
            (x * 255 / width.max(1)) as u8,
            (y * 255 / height.max(1)) as u8,
            128,
        ])
    })
}

fn corners_image() -> RgbImage {
    let mut image = RgbImage::new(2, 2);
    image.put_pixel(0, 0, Rgb([255, 0, 0]));
    image.put_pixel(1, 0, Rgb([0, 255, 0]));
    image.put_pixel(0, 1, Rgb([0, 0, 255]));
    image.put_pixel(1, 1, Rgb([255, 255, 255]));
    image
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

fn sample_dynamic(format: image::ImageFormat) -> DynamicImage {
    let rgb = gradient_image(6, 4);
    if format == image::ImageFormat::Ico {
        DynamicImage::ImageRgba8(DynamicImage::ImageRgb8(rgb).to_rgba8())
    } else {
        DynamicImage::ImageRgb8(rgb)
    }
}

fn convert_fixture(
    dir: &Path,
    input_name: &str,
    input_format: image::ImageFormat,
    output_name: &str,
    output_format: ImageFormat,
) -> (image::ImageFormat, DynamicImage) {
    let input = write_fixture(dir, input_name, &sample_dynamic(input_format), input_format);
    let output = dir.join(output_name);
    convert_and_reopen(
        &input,
        &output,
        output_format,
        &ConversionOptions::default(),
    )
}

fn transform_options(build: impl FnOnce(&mut TransformOptions)) -> ConversionOptions {
    let mut transform = TransformOptions::default();
    build(&mut transform);
    ConversionOptions {
        transform,
        encode: EncodeOptions::default(),
    }
}

macro_rules! roundtrip_tests {
    ($( $name:ident: $in_name:literal, $in_format:expr, $out_name:literal, $out_format:expr, $detected:expr );* $(;)?) => {
        $(
            #[test]
            fn $name() {
                let dir = temp_dir();
                let (format, image) = convert_fixture(&dir, $in_name, $in_format, $out_name, $out_format);
                assert_eq!(format, $detected);
                assert_eq!(image.dimensions(), (6, 4));
            }
        )*
    };
}

roundtrip_tests! {
    jpeg_to_png: "in.jpg", image::ImageFormat::Jpeg, "out.png", ImageFormat::Png, image::ImageFormat::Png;
    png_to_jpeg: "in.png", image::ImageFormat::Png, "out.jpg", ImageFormat::Jpeg, image::ImageFormat::Jpeg;
    png_to_webp: "in.png", image::ImageFormat::Png, "out.webp", ImageFormat::Webp, image::ImageFormat::WebP;
    webp_to_png: "in.webp", image::ImageFormat::WebP, "out.png", ImageFormat::Png, image::ImageFormat::Png;
    jpeg_to_avif: "in.jpg", image::ImageFormat::Jpeg, "out.avif", ImageFormat::Avif, image::ImageFormat::Avif;
    png_to_avif: "in.png", image::ImageFormat::Png, "out.avif", ImageFormat::Avif, image::ImageFormat::Avif;
    tiff_to_png: "in.tiff", image::ImageFormat::Tiff, "out.png", ImageFormat::Png, image::ImageFormat::Png;
    bmp_to_png: "in.bmp", image::ImageFormat::Bmp, "out.png", ImageFormat::Png, image::ImageFormat::Png;
    ico_to_png: "in.ico", image::ImageFormat::Ico, "out.png", ImageFormat::Png, image::ImageFormat::Png;
    png_to_tiff: "in.png", image::ImageFormat::Png, "out.tiff", ImageFormat::Tiff, image::ImageFormat::Tiff;
    png_to_bmp: "in.png", image::ImageFormat::Png, "out.bmp", ImageFormat::Bmp, image::ImageFormat::Bmp;
    png_to_ico: "in.png", image::ImageFormat::Png, "out.ico", ImageFormat::Ico, image::ImageFormat::Ico;
}

fn convert_corners(dir: &Path, name: &str, options: &ConversionOptions) -> RgbImage {
    let input = write_fixture(
        dir,
        &format!("{name}.png"),
        &DynamicImage::ImageRgb8(corners_image()),
        image::ImageFormat::Png,
    );
    let output = dir.join(format!("{name}-out.png"));
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, options);
    image.to_rgb8()
}

#[test]
fn rotation_90_swaps_dimensions() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| t.rotation = 90);
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (4, 6));
}

#[test]
fn rotation_90_moves_bottom_left_to_top_left() {
    let dir = temp_dir();
    let options = transform_options(|t| t.rotation = 90);
    let image = convert_corners(&dir, "rot90", &options);
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 255]);
}

#[test]
fn rotation_180_moves_bottom_right_to_top_left() {
    let dir = temp_dir();
    let options = transform_options(|t| t.rotation = 180);
    let image = convert_corners(&dir, "rot180", &options);
    assert_eq!(image.dimensions(), (2, 2));
    assert_eq!(image.get_pixel(0, 0).0, [255, 255, 255]);
}

#[test]
fn rotation_270_moves_top_right_to_top_left() {
    let dir = temp_dir();
    let options = transform_options(|t| t.rotation = 270);
    let image = convert_corners(&dir, "rot270", &options);
    assert_eq!(image.get_pixel(0, 0).0, [0, 255, 0]);
}

#[test]
fn invalid_rotation_is_rejected() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(corners_image()),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| t.rotation = 45);
    let result = convert_image(&input, &output, ImageFormat::Png, &options);
    assert!(matches!(result, Err(ConversionError::InvalidTransform(_))));
    assert!(!output.exists());
}

#[test]
fn horizontal_flip_moves_top_right_to_top_left() {
    let dir = temp_dir();
    let options = transform_options(|t| t.flip_horizontal = true);
    let image = convert_corners(&dir, "fliph", &options);
    assert_eq!(image.get_pixel(0, 0).0, [0, 255, 0]);
}

#[test]
fn vertical_flip_moves_bottom_left_to_top_left() {
    let dir = temp_dir();
    let options = transform_options(|t| t.flip_vertical = true);
    let image = convert_corners(&dir, "flipv", &options);
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 255]);
}

#[test]
fn resize_exact_changes_dimensions() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.resize = Some(ResizeOptions {
            width: 12,
            height: 8,
            exact: true,
        })
    });
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (12, 8));
}

#[test]
fn resize_preserves_aspect_ratio_when_not_exact() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.resize = Some(ResizeOptions {
            width: 3,
            height: 100,
            exact: false,
        })
    });
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (3, 2));
}

#[test]
fn crop_extracts_region() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.crop = Some(CropOptions {
            x: 1,
            y: 1,
            width: 3,
            height: 2,
        })
    });
    let (_, image) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
    assert_eq!(image.dimensions(), (3, 2));
    let expected = gradient_image(6, 4).get_pixel(1, 1).0;
    assert_eq!(image.to_rgb8().get_pixel(0, 0).0, expected);
}

#[test]
fn crop_out_of_bounds_is_rejected() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.crop = Some(CropOptions {
            x: 5,
            y: 3,
            width: 4,
            height: 4,
        })
    });
    let result = convert_image(&input, &output, ImageFormat::Png, &options);
    assert!(matches!(result, Err(ConversionError::InvalidTransform(_))));
    assert!(!output.exists());
}

#[test]
fn zero_size_crop_is_rejected() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.crop = Some(CropOptions {
            x: 0,
            y: 0,
            width: 0,
            height: 2,
        })
    });
    let result = convert_image(&input, &output, ImageFormat::Png, &options);
    assert!(matches!(result, Err(ConversionError::InvalidTransform(_))));
}

#[test]
fn zero_size_resize_is_rejected() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.resize = Some(ResizeOptions {
            width: 0,
            height: 4,
            exact: true,
        })
    });
    let result = convert_image(&input, &output, ImageFormat::Png, &options);
    assert!(matches!(result, Err(ConversionError::InvalidTransform(_))));
}

#[test]
fn oversized_resize_is_rejected() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.png");
    let options = transform_options(|t| {
        t.resize = Some(ResizeOptions {
            width: 20000,
            height: 20000,
            exact: true,
        })
    });
    let result = convert_image(&input, &output, ImageFormat::Png, &options);
    assert!(matches!(result, Err(ConversionError::InvalidTransform(_))));
}

#[test]
fn extension_normalization_maps_equivalents() {
    assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
    assert_eq!(ImageFormat::from_extension("JPEG"), Some(ImageFormat::Jpeg));
    assert_eq!(ImageFormat::from_extension("tif"), Some(ImageFormat::Tiff));
    assert_eq!(ImageFormat::from_extension("tiff"), Some(ImageFormat::Tiff));
    assert_eq!(ImageFormat::from_extension(".png"), Some(ImageFormat::Png));
    assert_eq!(ImageFormat::from_extension("svg"), None);
    assert_eq!(ImageFormat::from_extension("gif"), None);
}

#[test]
fn unsupported_svg_target_format_is_rejected() {
    let result = serde_json::from_str::<ImageFormat>("\"svg\"");
    assert!(result.is_err());
    assert!(result
        .err()
        .map(|e| e.to_string().contains("unsupported image format"))
        .unwrap_or(false));
}

#[test]
fn garbage_bytes_are_rejected_as_unsupported() {
    let dir = temp_dir();
    let output = dir.join("out.png");
    let result = convert_bytes(
        b"this is not an image",
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(matches!(
        result,
        Err(ConversionError::UnsupportedInputFormat(_))
    ));
    assert!(!output.exists());
}

#[test]
fn missing_input_file_is_reported() {
    let dir = temp_dir();
    let input = dir.join("does-not-exist.png");
    let output = dir.join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(matches!(result, Err(ConversionError::InputNotFound(_))));
}

#[test]
fn output_in_missing_directory_is_reported() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("missing-dir").join("out.png");
    let result = convert_image(
        &input,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    );
    assert!(matches!(result, Err(ConversionError::OutputPath(_))));
}

#[test]
fn input_file_is_not_modified() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let before = fs::read(&input).expect("could not read input");
    let output = dir.join("out.jpg");
    convert_image(
        &input,
        &output,
        ImageFormat::Jpeg,
        &ConversionOptions::default(),
    )
    .expect("conversion failed");
    let after = fs::read(&input).expect("could not read input");
    assert_eq!(before, after);
}

#[test]
fn content_detection_overrides_wrong_extension() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "actually-png.jpg",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.webp");
    let (format, image) = convert_and_reopen(
        &input,
        &output,
        ImageFormat::Webp,
        &ConversionOptions::default(),
    );
    assert_eq!(format, image::ImageFormat::WebP);
    assert_eq!(image.dimensions(), (6, 4));
}

#[test]
fn jpeg_quality_range_is_validated() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.jpg");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            jpeg_quality: Some(0),
            ..EncodeOptions::default()
        },
    };
    let result = convert_image(&input, &output, ImageFormat::Jpeg, &options);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));
    assert!(!output.exists());
}

#[test]
fn png_compression_levels_all_encode() {
    for level in [
        PngCompression::Fast,
        PngCompression::Balanced,
        PngCompression::Maximum,
    ] {
        let dir = temp_dir();
        let input = write_fixture(
            &dir,
            "in.png",
            &DynamicImage::ImageRgb8(gradient_image(6, 4)),
            image::ImageFormat::Png,
        );
        let output = dir.join("out.png");
        let options = ConversionOptions {
            transform: TransformOptions::default(),
            encode: EncodeOptions {
                png_compression: Some(level),
                ..EncodeOptions::default()
            },
        };
        let (format, _) = convert_and_reopen(&input, &output, ImageFormat::Png, &options);
        assert_eq!(format, image::ImageFormat::Png);
    }
}

fn jpeg_with_exif_orientation(orientation: u16) -> Vec<u8> {
    let mut plain = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(gradient_image(4, 2))
        .write_with_encoder(JpegEncoder::new_with_quality(&mut plain, 90))
        .expect("could not encode jpeg fixture");
    let jpeg = plain.into_inner();
    assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]);

    let mut exif = Vec::new();
    exif.extend_from_slice(b"Exif\0\0");
    exif.extend_from_slice(b"II");
    exif.extend_from_slice(&42u16.to_le_bytes());
    exif.extend_from_slice(&8u32.to_le_bytes());
    exif.extend_from_slice(&1u16.to_le_bytes());
    exif.extend_from_slice(&0x0112u16.to_le_bytes());
    exif.extend_from_slice(&3u16.to_le_bytes());
    exif.extend_from_slice(&1u32.to_le_bytes());
    exif.extend_from_slice(&u32::from(orientation).to_le_bytes());
    exif.extend_from_slice(&0u32.to_le_bytes());

    let segment_length = (exif.len() + 2) as u16;
    let mut result = Vec::with_capacity(jpeg.len() + exif.len() + 4);
    result.extend_from_slice(&jpeg[0..2]);
    result.extend_from_slice(&[0xFF, 0xE1]);
    result.extend_from_slice(&segment_length.to_be_bytes());
    result.extend_from_slice(&exif);
    result.extend_from_slice(&jpeg[2..]);
    result
}

#[test]
fn exif_orientation_is_applied_on_decode() {
    let dir = temp_dir();
    let data = jpeg_with_exif_orientation(6);
    let output = dir.join("out.png");
    let converted = convert_bytes(
        &data,
        &output,
        ImageFormat::Png,
        &ConversionOptions::default(),
    )
    .expect("conversion failed");
    assert_eq!((converted.width, converted.height), (2, 4));
}

#[test]
fn exif_orientation_and_user_rotation_compose() {
    let dir = temp_dir();
    let data = jpeg_with_exif_orientation(6);
    let output = dir.join("out.png");
    let options = transform_options(|t| t.rotation = 90);
    let converted =
        convert_bytes(&data, &output, ImageFormat::Png, &options).expect("conversion failed");
    assert_eq!((converted.width, converted.height), (4, 2));
}

fn noise_image(size: u32) -> RgbImage {
    let mut state: u32 = 0x1234_5678;
    let mut next = move || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 16) as u8
    };
    ImageBuffer::from_fn(size, size, |_, _| Rgb([next(), next(), next()]))
}

#[test]
fn webp_lossy_quality_encodes() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.webp");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            webp_quality: Some(60),
            ..EncodeOptions::default()
        },
    };
    let (format, image) = convert_and_reopen(&input, &output, ImageFormat::Webp, &options);
    assert_eq!(format, image::ImageFormat::WebP);
    assert_eq!(image.dimensions(), (6, 4));
}

#[test]
fn webp_lossy_preserves_alpha() {
    let dir = temp_dir();
    let rgba: image::RgbaImage = ImageBuffer::from_fn(8, 8, |x, y| {
        image::Rgba([(x * 30) as u8, (y * 30) as u8, 200, 255 - (x * 20) as u8])
    });
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgba8(rgba),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.webp");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            webp_quality: Some(80),
            ..EncodeOptions::default()
        },
    };
    let (format, image) = convert_and_reopen(&input, &output, ImageFormat::Webp, &options);
    assert_eq!(format, image::ImageFormat::WebP);
    assert!(image.color().has_alpha());
    assert_eq!(image.dimensions(), (8, 8));
}

#[test]
fn webp_quality_range_is_validated() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.webp");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            webp_quality: Some(0),
            ..EncodeOptions::default()
        },
    };
    let result = convert_image(&input, &output, ImageFormat::Webp, &options);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));
    assert!(!output.exists());
}

#[test]
fn webp_lossy_is_smaller_than_lossless_for_noise() {
    let image = DynamicImage::ImageRgb8(noise_image(64));
    let lossless = convertia_lib::conversion::encoder::encode_to_vec(
        &image,
        ImageFormat::Webp,
        &EncodeOptions::default(),
    )
    .expect("lossless encode failed");
    let lossy = convertia_lib::conversion::encoder::encode_to_vec(
        &image,
        ImageFormat::Webp,
        &EncodeOptions {
            webp_quality: Some(20),
            ..EncodeOptions::default()
        },
    )
    .expect("lossy encode failed");
    assert!(lossy.len() < lossless.len());
}

#[test]
fn avif_custom_quality_and_speed_encode() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.avif");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            avif_quality: Some(70),
            avif_speed: Some(8),
            ..EncodeOptions::default()
        },
    };
    let (format, image) = convert_and_reopen(&input, &output, ImageFormat::Avif, &options);
    assert_eq!(format, image::ImageFormat::Avif);
    assert_eq!(image.dimensions(), (6, 4));
}

#[test]
fn avif_quality_range_is_validated() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.avif");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            avif_quality: Some(0),
            ..EncodeOptions::default()
        },
    };
    let result = convert_image(&input, &output, ImageFormat::Avif, &options);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));
    assert!(!output.exists());
}

#[test]
fn avif_speed_range_is_validated() {
    let dir = temp_dir();
    let input = write_fixture(
        &dir,
        "in.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let output = dir.join("out.avif");
    let options = ConversionOptions {
        transform: TransformOptions::default(),
        encode: EncodeOptions {
            avif_speed: Some(11),
            ..EncodeOptions::default()
        },
    };
    let result = convert_image(&input, &output, ImageFormat::Avif, &options);
    assert!(matches!(result, Err(ConversionError::InvalidOptions(_))));
    assert!(!output.exists());
}

#[test]
fn batch_conversion_with_paths() {
    use convertia_lib::commands::convert::{convert_batch, ConvertImagesRequest, SourceImage};

    let dir = temp_dir();
    let first = write_fixture(
        &dir,
        "first.png",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Png,
    );
    let second = write_fixture(
        &dir,
        "second.jpg",
        &DynamicImage::ImageRgb8(gradient_image(6, 4)),
        image::ImageFormat::Jpeg,
    );
    let missing = dir.join("missing.png");
    let output_dir = dir.join("out");
    fs::create_dir_all(&output_dir).expect("could not create output directory");

    let request = ConvertImagesRequest {
        images: vec![
            SourceImage {
                path: first.to_string_lossy().into_owned(),
                rotation: 90,
            },
            SourceImage {
                path: second.to_string_lossy().into_owned(),
                rotation: 0,
            },
            SourceImage {
                path: missing.to_string_lossy().into_owned(),
                rotation: 0,
            },
        ],
        target_format: ImageFormat::Webp,
        jpeg_quality: None,
        png_compression: None,
        webp_quality: Some(75),
        avif_quality: None,
        avif_speed: None,
    };

    let response = convert_batch(&request, &output_dir);
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.output_directory, output_dir.to_string_lossy());

    let first_result = &response.results[0];
    assert_eq!(first_result.source_name, "first.png");
    assert!(first_result.error.is_none());
    let first_output = first_result
        .output_path
        .as_ref()
        .expect("missing output path");
    assert!(first_output.ends_with("first.webp"));
    let rotated = ImageReader::open(first_output)
        .expect("could not open output")
        .decode()
        .expect("could not decode output");
    assert_eq!(rotated.dimensions(), (4, 6));

    let second_result = &response.results[1];
    assert!(second_result.error.is_none());
    assert!(second_result
        .output_path
        .as_ref()
        .expect("missing output path")
        .ends_with("second.webp"));

    let third_result = &response.results[2];
    assert!(third_result.output_path.is_none());
    assert!(third_result
        .error
        .as_ref()
        .expect("missing error")
        .contains("Input file not found"));
}
