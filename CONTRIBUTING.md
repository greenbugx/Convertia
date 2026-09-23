# Contributing to Convertia

Thanks for helping improve Convertia. This document covers how to work on the project and what a good contribution looks like.

## Project scope

Convertia is a local, offline conversion toolkit. Contributions must keep these guarantees:

- No network requests, no uploads, no telemetry, no remote processing
- Source files are never modified or overwritten
- No `PDF`, documents, OCR, audio, or video unless it is an agreed direction described in [UNSUPPORTED.md](UNSUPPORTED.md)
- New features start in the frontend contract and the backend follows it, not the other way around

Before building something large, open an issue describing the change so scope can be agreed first.

## Development setup

Requirements: Rust, Node.js, pnpm, and the Tauri 2 system dependencies for your platform.

```
pnpm install
pnpm tauri:dev
```

## Code style

Rust:

- Run `cargo fmt` before committing, and `cargo clippy --all-targets -- -D warnings` must pass
- Use Result based error handling. No unwrap or expect in production code paths
- No unsafe Rust
- No unnecessary comments in source code
- Keep modules small and focused. Do not add abstractions without a real benefit
- Conversion logic stays framework independent. Keep Tauri command code thin

TypeScript:

- Follow the existing component and file conventions
- Keep the frontend contract in sync with the Rust request and response types

General:

- No strings, error messages, documentation, or commit messages
- Explicit, readable names over short names

## Tests

- Backend changes need tests in `src-tauri/tests`
- Tests must run without the Tauri window, using generated or fixture data only
- New formats and new transformations need roundtrip and error case coverage
- Vectorization changes need config mapping coverage for presets, sliders, and color mode overrides, plus a roundtrip that asserts the output contains vector paths and no embedded raster
- Run the suite before pushing:

```
cd src-tauri
cargo test
```

## Commits

Conventional commit style, lowercase, imperative mood:

```
feat: add support for something
fix: correct something
chore: update something
docs: clarify something
refactor: simplify something
perf: improve speed of something
build: update build configuration
ci: update continuous integration checks
revert: undo a previous change
test: add tests for something
```

Keep the subject line short and specific.

## Pull requests

- Branch from `main` and keep changes focused, one concern per pull request
- Update [SUPPORTED.md](SUPPORTED.md) and [UNSUPPORTED.md](UNSUPPORTED.md) when behavior changes
- CI runs `cargo fmt`, clippy with warnings denied, the test suite, and a release build. It must be green before merge
- Describe what changed and how it was tested

## Reporting bugs

Open an issue with the input format, output format, the options you used, and what you expected versus what happened. Attach a sample file if you can, or a minimal `SVG` or generated description of the image if the file is sensitive.
