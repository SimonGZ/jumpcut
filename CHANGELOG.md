# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - Unreleased

### Added
- Added first-class PDF output, including tagged-PDF structure, XMP metadata, author metadata, and page labels that follow visible screenplay numbering.
- Added shared layout-profile support for A4 page geometry, page-margin overrides, and lines-per-page overrides across PDF, HTML, and FDX output.
- Added an explicit `--write` / `-w` CLI mode for auto-derived output paths while keeping stdout as the default.
- Added `--no-title-page` support for HTML and PDF output.
- Added a dedicated formatting and metadata reference document at [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).
- Added readable long-form `fmt` aliases such as `bold-scene-headings`, `underline-scene-headings`, and `all-caps-action`.
- Added a clearer user-facing WASM packaging entrypoint under `scripts/wasm/generate-package.sh`.
- Added a feature-gated PDF export to the wasm wrapper and extended the wasm size/runtime report to cover `pdf_only`.

### Changed
- Promoted the project to a `1.0.0-beta` baseline in crate metadata while preparing for the final `1.0.0` release.
- Renamed the default render profile from `final-draft` to `industry`.
- Reworked paginated HTML so page-size/layout overrides flow through the shared layout profile instead of being hard-coded to Letter.
- Derived PDF title-page vertical placement from shared geometry instead of a separate letter-specific coordinate system.
- Split README-heavy formatting and metadata material into a dedicated reference doc and rewrote that guidance in more user-facing language.
- Separated normal wasm package generation from the older research-era script naming.

### Fixed
- Fixed wasm package generation so the wrapper no longer pulled in native-only diagnostics code.
- Fixed wasm PDF generation on `wasm32-unknown-unknown` by removing its dependency on a native wall-clock source.
- Fixed headless paginated HTML fragments so they carry the layout CSS they need for A4 and other page-geometry overrides.
- Fixed feature-boundary issues that produced large `dead_code` warning clusters in non-HTML/non-PDF wasm slices.

## [1.0.0-alpha.1] - 2026-03-31

### Added (Major Rewrite)
- Initial alpha release of the major rewrite focusing on geometry-driven fractional pagination.
- Transitioned pagination engine to rely exclusively on `LayoutGeometry` for layout measurements.
- Implemented robust, space-preserving greedy wrapping algorithm for dialogue and action blocks.
- Added parity-harness verification tools for Big Fish, Little Women, and Mostly Genius screenplay baselines.

## [0.8.1] - 2026-03-22

### Fixed
- Fixed parser handling for leading indentation in dialogue, parentheticals, and transitions.
- Added regression tests covering indented dialogue blocks and related whitespace-sensitive cases.

## [0.8.0] - 2026-03-22

### Added
- Added the `jumpcut-wasm` wrapper crate and documented how to build and use the wasm package.
- Added wasm benchmarking and verification scripts for size and runtime tracking.
- Added regression coverage for Unicode styling, invisible characters, and parser boundary handling.

### Changed
- Replaced production Handlebars rendering with handwritten HTML and FDX emitters.
- Reorganized rendering code into explicit HTML, FDX, and shared modules.
- Reduced wasm dependency and binary size substantially through regex, Unicode, and renderer cleanup.
- Preserved internal invisible characters and emoji/ZWJ sequences in screenplay text while keeping parser classifiers robust.

### Fixed
- Fixed parser behavior so leading invisible characters no longer break metadata and scene-heading recognition without mutating internal text content.

## [0.7.5] - 2025-07-10

### Fixed
- Fixing parser bug where odd numbers of blank lines were not handled correctly.

## [0.7.4] - 2025-07-11

### Added
- Adding metadata CLI command

## [0.7.3]

### Added
- Adding new metadata fmt support for arbitrary dialogue margins.

## [0.7.2]

### Fixed
- Fixing parser bug where any word ending in "to:" became a transition.

## [0.7.1]

### Changed
- Improving documentation.

## [0.7.0]

### Added
- Initial public release. Supports FDX, HTML, and JSON output.
