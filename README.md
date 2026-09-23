# Convertia

Convertia is a local, offline conversion toolkit for your desktop, built with Tauri 2, Rust, and React. The first tool in the suite is image conversion: select images, pick an output format, optionally set quality options, convert. Everything runs on your machine. Nothing is uploaded anywhere.

Convertia is being built as a collection of conversion tools rather than a single purpose app. The current version focuses on raster and SVG image conversion, and the core architecture is designed so more tools can be added without rewriting the existing pipeline.

- [SUPPORTED.md](SUPPORTED.md): everything that works in the current version
- [UNSUPPORTED.md](UNSUPPORTED.md): what is not supported yet, and whether it is planned, possible, or out of scope for the future

## The image conversion tool

Inputs: `JPEG`, `JPG`, `PNG`, `WebP`, `AVIF`, `TIFF`, `TIF`, `ICO`, `BMP`, `SVG`
Outputs: `JPEG`, `JPG`, `PNG`, `WebP`, `AVIF`, `TIFF`, `TIF`, `ICO`, `BMP`, `SVG`

`SVG` input is rasterized with `usvg` and `resvg` and then follows the same pipeline as raster inputs. Raster inputs are traced into `SVG` output with VTracer, producing real vector paths rather than an embedded raster. `SVG` to `SVG` is not supported.

### Conversion options

- Per image rotation: `0`, `90`, `180`, `270` degrees clockwise
- JPEG quality: `1` to `100`, default `85`
- PNG compression: `Fast`, `Balanced`, `Maximum`
- WebP quality: `1` to `100`, lossy encoding through `libwebp`
- AVIF quality: `1` to `100` and speed: `1` to `10`
- Vectorization preset: `Logo`, `Photo`, `Black & White`, `Poster`
- Vectorization color mode: `Color` or `Black & White`
- Vectorization detail: `1` to `100`
- Vectorization smoothness: `1` to `100`
- Vectorization color detail: `1` to `100`, shown only in Color mode

Transparency is preserved for `PNG`, `WebP`, `AVIF`, and `TIFF` output. `JPEG` output flattens transparency onto a white background. `ICO` output is capped at 256x256 pixels. Traced `SVG` output keeps transparent regions unpainted through VTracer's transparency keying instead of turning them into an opacity layer.

### How it works

The conversion core is a decode, transform, encode pipeline:

1. Input detection routes the file to the raster decoder or the `SVG` decoder
2. Both produce a single image crate DynamicImage
3. The common transformer applies rotation, flips, crop, and resize with validation
4. The encoder writes the selected raster format, or the vectorizer traces the transformed image into SVG paths with VTracer

EXIF orientation is normalized for raster inputs before user transforms. `SVG` files have no EXIF data, so they skip that step.

### Features

- Native file picker and drag and drop
- Batch conversion with per image results
- Thumbnails load through a custom preview scheme. The app grants read access only to files selected in the current session and revokes access when an image is removed from the list.
- Output files are written to `Pictures/Convertia` with deduplicated names. Existing files and source images are never overwritten.

## Development

Requirements: Rust, Node.js, pnpm, and the Tauri 2 system dependencies for your platform.

```
pnpm install
pnpm tauri:dev
pnpm tauri:build
pnpm build
```

## Backend tests

```
cd src-tauri
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The suite covers format roundtrips, transformations, error handling, EXIF orientation, `SVG` rendering, raster to `SVG` tracing, vectorization option mapping, and preview access grants.

## Continuous integration

`.github/workflows` contains three workflows that share the same ignore list, so documentation only changes trigger nothing:

- `backend-ci` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` on the Rust side only
- `frontend-ci` runs the TypeScript type check (`tsc --noEmit`) and a Vite bundle of the web interface
- `build-ci` starts after `backend-ci` and `frontend-ci` finish. It waits for both to complete for the same commit, requires both to be green, and then performs the full `pnpm tauri build` release build as the final gate. A red check on either side blocks the build.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, style rules, and the pull request process.

## Project layout

- `src-tauri/src/conversion`: format definitions, options, decoder, transformer, encoder, `SVG` decoder, vectorizer, vectorization config, output writing, orchestration, errors
- `src-tauri/src/commands`: thin Tauri command layer
- `src-tauri/tests`: conversion, `SVG`, vectorization, and preview test suites
- `src`: React and TypeScript interface

## License

Released under the MIT License. See [LICENSE](LICENSE).


