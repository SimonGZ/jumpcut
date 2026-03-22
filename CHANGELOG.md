# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
