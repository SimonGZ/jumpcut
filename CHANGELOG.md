# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Add `Fmt` left/right edge tags for action (`al`/`ar`), scene headings (`shl`/`shr`), character cues (`cl`/`cr`), parentheticals (`pl`/`pr`), transitions (`tl`/`tr`), lyrics (`ll`/`lr`), cold openings (`col`/`cor`), new acts (`nal`/`nar`), and end-of-act headings (`eal`/`ear`), alongside existing dialogue tags (`dl`/`dr`).
- Preserve imported element and page margins in Fountain exports using `Fmt` tags.
- Honor element edges in ordinary, exact-wrap, and paginated HTML, including fractional offsets and centered headings. Preserve fractional margins in FDX exports and character-cue positions in PDF.

## [2.0.1] - 2026-09-12

### Fixed
- Right-align HTML transitions such as `CUT TO:` at the same position relative to action text as PDF output, using the layout profile's transition margin and the PDF character grid in normal, exact-wrap, and paginated output.
- Match HTML body character spacing to the PDF's fixed 7-point advance, keeping text alignment consistent with physical screenplay margins. This can change browser-selected line breaks in normal HTML.
- Add an opt-in browser regression test that measures transition placement, including styled and wrapped transitions and responsive normal HTML.

## [2.0.0] - 2026-09-11

### Changed
- I've rethought the two pagination profiles. The default which includes my personal aesthetic tweaks is now just called `jumpcut`.
- `industry` is now the name of the profile that closely emulates Final Draft-style pagination. This profile has been rewritten and should now provide PDF pagination that almost exactly matches Final Draft, even in cases where I think Final Draft's choices are ugly.
- Breaking Rust API changes: pagination configuration now uses `PaginationPolicy`; public pagination functions, fields, and wrapping names have changed. Callers using these APIs must migrate to the new policy-based interfaces.
- Renamed the FDX serialization API to `Screenplay::to_fdx()`, retaining `to_final_draft()` as a deprecated alias.

## [1.0.0] - 2026-08-02

### Added
- Added first-class PDF output, including tagged-PDF structure, XMP metadata, author metadata, and page labels that follow visible screenplay numbering.
- Added shared layout-profile support for A4 page geometry, page-margin overrides, and lines-per-page overrides across PDF, HTML, and FDX output.
- Added shared per-element layout overrides ("cheats") across Fountain and FDX to override spacing for individual elements.
- Added Fountain layout modifier classes to Fountain notes (`.lift` and `.widen`) to make it easier to hand tweak individual element margins and spacing.
- Added readable long-form `fmt` aliases such as `bold-scene-headings`, `underline-scene-headings`, and `all-caps-action`.
- Added a dedicated formatting and metadata reference document at [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).
- Added an explicit `--write` / `-w` CLI mode for auto-derived output paths while keeping stdout as the default.
- Added `--no-title-page` support for HTML and PDF output.
- Added PDF scene bookmarks that navigate directly to scene headings.
- Added `--font-dir` support for custom four-face TrueType families in PDF output and embedded-font HTML output.
- Added a clearer user-facing WASM packaging entrypoint under `scripts/wasm/generate-package.sh`.
- Added a feature-gated PDF export to the wasm wrapper and extended the wasm size/runtime report to cover `pdf_only`.
- Added an opt-in `pdf-font-subsetting` wasm feature and a corresponding `pdf_subset` size-report slice.

### Changed
- Stabilized the crate and CLI metadata for the final `1.0.0` release.
- Renamed the default render profile from `final-draft` to `industry`.
- Reworked paginated HTML so page-size/layout overrides flow through the shared layout profile instead of being hard-coded to Letter.
- Derived PDF title-page vertical placement from shared geometry instead of a separate letter-specific coordinate system.
- Split README-heavy formatting and metadata material into a dedicated reference doc and rewrote that guidance in more user-facing language.
- Separated normal wasm package generation from the older research-era script naming.
- Preserved paragraph-level `SpaceBefore` and `RightIndent` deviations through FDX import/export and structural act-break upgrades instead of flattening them into only document-level defaults.
- Kept lossless PDF compression in the default wasm build while reserving font subsetting for native defaults and explicit wasm opt-in, avoiding roughly 606 KB of raw wasm in the default package.

### Fixed
- Fixed wasm package generation so the wrapper no longer pulled in native-only diagnostics code.
- Fixed wasm PDF generation on `wasm32-unknown-unknown` by removing its dependency on a native wall-clock source.
- Fixed standalone WASM feature builds so FDX-only builds no longer reference disabled HTML and PDF APIs.
- Fixed the WASM packaging helper so its `wasm-bindgen-cli` version follows the Rust crate version in `Cargo.lock`.
- Fixed Cargo feature boundaries so library-only builds skip the CLI and optional renderer dependencies.
- Fixed the CLI so unsupported output formats produce an argument error instead of successful placeholder output.
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
