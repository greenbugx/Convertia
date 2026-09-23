# Supported

Everything listed here works in the current version of Convertia and is covered by the test suite or verified in the app.

## Image conversion

### Input formats

- JPEG (`.jpg`, `.jpeg`)
- PNG (`.png`)
- WebP (`.webp`)
- AVIF (`.avif`)
- TIFF (`.tif`, `.tiff`)
- ICO (`.ico`)
- BMP (`.bmp`)
- SVG (`.svg`)

Equivalent extensions are normalized internally: `jpg` and `jpeg` map to `JPEG`, `tif` and `tiff` map to `TIFF`. Raster inputs are detected from file content first, so a `PNG` renamed to `.jpg` still converts correctly. `SVG` is routed by extension and parsed with `usvg`.

### Output formats

- JPEG (`.jpg`)
- PNG (`.png`)
- WebP (`.webp`)
- AVIF (`.avif`)
- TIFF (`.tiff`)
- ICO (`.ico`)
- BMP (`.bmp`)
- SVG (`.svg`)

Every raster input converts to every output format, including traced `SVG`. `SVG` input converts to every raster output. `SVG` to `SVG` is not supported.

### Transformations

- Per image rotation: `0`, `90`, `180`, `270` degrees clockwise
- EXIF orientation normalization for raster inputs, applied before user rotation

The backend core also implements horizontal flip, vertical flip, crop, and resize with validation, and these are covered by tests. They are not exposed in the user interface yet. See [UNSUPPORTED.md](UNSUPPORTED.md).

### Quality and compression options

- JPEG quality: `1` to `100`, default `85`
- PNG compression: `Fast`, `Balanced`, `Maximum` (mapped to `PNG` compression levels, not a quality number)
- WebP quality: `1` to `100`, real lossy encoding through `libwebp`
- AVIF quality: `1` to `100` and speed: `1` to `10`

### Transparency behavior

- `PNG`, `WebP`, `AVIF`, and `TIFF` outputs preserve alpha
- `JPEG` output flattens transparency onto a white background
- `BMP` preserves alpha through its RGBA path
- `ICO` output always uses an RGBA entry, as required by the `ICO` format
- `ICO` output is capped at 256x256 pixels

### SVG input

- `SVG` is rasterized with `usvg` and `resvg`, then follows the exact same transform and encode pipeline as raster inputs
- Natural size comes from width, height, or viewBox, with the documented 100x100 fallback only when no valid size is declared
- Invalid declared dimensions are rejected instead of silently clamped
- Render size is validated against a pixel cap before allocation, so a malformed `SVG` cannot cause an uncontrolled memory allocation
- Relative local resources, such as linked images in the same folder, resolve against the SVG file's directory
- Text renders with fonts available to the system font database

### Raster to SVG vectorization

Raster inputs are traced into `SVG` with VTracer `0.6.5`, producing real vector paths. The trace runs on the already decoded and already transformed image, so EXIF normalization and per image rotation apply before tracing, and no file is decoded twice.

Options exposed in the interface:

- Preset: `Logo`, `Photo`, `Black & White`, `Poster`. `Photo`, `Black & White`, and `Poster` use VTracer's built in baselines. `Logo` is a Convertia specific configuration derived from the Poster baseline for cleaner compact paths.
- Color mode: `Color` or `Black & White`. The user's choice always overrides the mode the preset would otherwise apply.
- Detail: `1` to `100`. Low discards more tiny regions and simplifies geometry for smaller output. High preserves more regions and geometry for larger output.
- Smoothness: `1` to `100`. Low keeps sharper corners and more local structure. High produces rounder curves with fewer sharp transitions.
- Color detail: `1` to `100`, visible only in Color mode. Low merges colors into fewer layers. High preserves more color layers. It is ignored completely in Black & White mode.
- VTracer's raw fields, including hierarchical mode, curve fitting mode, speckle filter, color precision, layer difference, thresholds, and path precision, stay internal and are never sent from the interface.

Notes on traced output:

- Tracing is an approximation of the source raster. Original vector structure, layers, or text are never recovered.
- Fully transparent pixels are keyed and dropped the way VTracer documents it, so transparent regions stay unpainted instead of becoming a per pixel opacity layer.
- Presets that already sit at VTracer's maximum color precision see the top of the Color detail slider converge on that maximum, because VTracer supports at most 8 significant bits per channel.

## Application behavior

- Native file picker and drag and drop
- Batch conversion with per image results: one failing file does not stop the batch
- Per image error messages shown in the interface
- Rotation is configured per image before conversion
- Outputs are written to `Pictures/Convertia`, falling back to Downloads, then the system temp folder
- Output names are sanitized and deduplicated, so existing files and source images are never overwritten
- First successful output is revealed in the system file manager
- Previews load through a custom preview scheme. Read access is granted only to files selected in the current session and is revoked when an image is removed from the list or the list is cleared.

## Privacy and security

- Fully local and offline. No network requests, no uploads, no telemetry.
- Source files are opened read only and are never modified
- The webview can only read files the user explicitly picked in the current session
- Malformed input files produce descriptive errors and no output file

## Quality gates

- 122 backend tests: format roundtrips, transformations, error cases, EXIF orientation, `SVG` rendering, raster to `SVG` tracing, vectorization option mapping, and preview access grants
- Continuous integration runs `cargo fmt`, `cargo clippy` with warnings denied, the full test suite, and a release build on every code change
