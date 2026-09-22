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

All supported conversions: every input format above converts to every output format above.

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

- 86 backend tests: format roundtrips, transformations, error cases, EXIF orientation, `SVG` rendering, and preview access grants
- Continuous integration runs `cargo fmt`, `cargo clippy` with warnings denied, the full test suite, and a release build on every code change
