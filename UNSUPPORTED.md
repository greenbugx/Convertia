# Unsupported

What Convertia does not do today, and whether that will change in the future. Status labels:

- Planned: will be added, the direction is decided
- Likely: expected to be added, not committed yet
- Under consideration: may be added, no commitment
- Not planned: out of scope on purpose, very unlikely to change

## User interface gaps

### Flip, crop, and custom resize controls

The backend core already implements horizontal flip, vertical flip, crop, and resize with validation and tests. The interface only exposes rotation today.

Status: Planned

### Custom output directory selection

Outputs always go to Pictures/Convertia, with fallbacks to Downloads and the temp folder. There is no picker for a custom destination yet.

Status: Planned

### Conversion progress during large batches

Results appear when the whole batch finishes. There is no live progress bar or per image status list while a large batch is running.

Status: Likely

### Resizable image preview

The preview area has a fixed layout. Zooming or enlarging previews is not available.

Status: Planned

## Image format gaps

### SVG to SVG

An `SVG` input cannot be re-traced into a new `SVG`. `SVG` input converts to every raster output, and raster input converts to `SVG`, but the combination is rejected with a clear error.

Status: Not planned

### Recovering original vector structure from a raster

Tracing always produces an approximation of the source raster. Original paths, layers, or text from a hand authored vector file are never recovered once it has been rasterized.

Status: Not planned and not achievable by any tracer

### Partial opacity in traced SVG output

VTracer keys fully transparent pixels instead of encoding per pixel opacity, so traced `SVG` contains opaque paths over unpainted transparent regions rather than partial alpha layers.

Status: Not planned, follows the VTracer design

### Raw VTracer controls in the interface

Hierarchical mode, curve fitting mode, speckle filter, color precision, layer difference, corner and splice thresholds, and path precision remain internal implementation details. The interface stays at the preset and slider abstraction on purpose.

Status: Likely

### SVGZ files (.svgz)

Gzip compressed SVG files are not accepted as a file type. The renderer can handle gzipped content, but the extension is not in the accepted list.

Status: Under consideration

### GIF input or output

GIF is not among the enabled codecs.

Status: Under consideration

### HEIC, HEIF, and camera RAW input

Not enabled as input codecs. Adding them increases binary size and dependencies.

Status: Under consideration

### Animated output

There is no animated WebP, GIF, or APNG output. Every conversion produces a single still frame.

Status: Not planned for now

## SVG feature gaps

### Animated SVG (SMIL and CSS animations)

Animation is not part of a static image. Conversion keeps the starting frame.

Status: Not planned, no static format can replace it

### SVG scripts (script elements)

Scripts inside SVG files are never executed. This is a security requirement, since SVG input is untrusted. Content that only exists because a script drew it will be missing from the output.

Status: Not planned, never executable by design

### foreignObject (embedded HTML)

HTML embedded inside an SVG is skipped by the renderer because rendering it would require a full HTML layout engine.

Status: Not planned

### Fonts not installed on the system

SVG text renders with fonts available to the system font database. Text referencing a font that is not installed falls back to the default font or renders without the intended glyphs.

Status: Likely, the app may bundle common fonts later

## Future tools

Convertia is designed as a conversion toolkit, not a single purpose app. These are the tool directions beyond image conversion. The pipeline architecture already separates decode, transform, and encode stages so new engines can reuse them.

### Images to PDF

Status: Planned as the next tool

### PDF to images

Status: Planned

### Document conversion (Office formats, text documents)

Status: Under consideration

### OCR (text recognition from images)

Status: Under consideration

### Audio conversion

Status: Under consideration

### Video conversion

Status: Under consideration

## Things that will never be supported

### Cloud, server, or account based processing

Convertia is local and offline by design. Files never leave the machine.

Status: Not planned, would break the core promise of the app

### Network based conversion or uploads

Same guarantee as above. No telemetry, no analytics, no remote processing.

Status: Not planned, would break the core promise of the app

### Overwriting source images

Convertia never modifies input files and never overwrites existing outputs.

Status: Not planned, would break a core safety guarantee
