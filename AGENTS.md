# Repository Guidelines

## Project Structure & Module Organization
Core parsing flows through `src/lib.rs`, with supporting logic in `converters.rs` and `text_style_parser.rs`. The CLI resides in `src/bin/main.rs` behind the `cli` feature. Templates for HTML/FDX output are under `src/templates/`. Regression tests live in `tests/`, named after the grammar areas they cover (e.g. `parse_notes_test.rs`). Criterion benchmarks sit in `benches/`, and `pkg/` holds wasm-pack output.

## Build, Test, and Development Commands
Use `cargo build` for iteration; `cargo build --release` mirrors the default `just` recipe. `cargo test` covers unit and integration suites, while `cargo test --all-features` guards feature combinations. Keep the tree lint-free with `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings`. WASM consumers should run `just wasm-browser` or `just wasm-node`, which invoke `wasm-pack build` with the `wasm` feature.

## Coding Style & Naming Conventions
Rustfmt defaults apply (4-space indent, trailing commas). Keep modules focused and choose filenames that mirror parser responsibilities (`parse_*`). Use `snake_case` for functions and modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Note relevant feature flags (`cli`, `html`, `fdx`, `wasm`) when adding docs or examples.

## Testing Guidelines
Co-locate unit tests with their modules and reserve `tests/` for parser integrations. Follow existing naming (`parse_scene_heading_test.rs`, etc.) and rely on `pretty_assertions::assert_eq!` for readable diffs. When altering parsing behavior, add fixtures that demonstrate the new or edge-case grammar. Always run `cargo test --all-features` before sending a PR.

## Commit & Pull Request Guidelines
Write concise, imperative commits ("Trim centered whitespace"), optionally prefixed (`feat:`, `fix:`) when helpful. Reference related issues in the body and update `CHANGELOG.md` for user-visible shifts. PRs should summarize scope, list verification commands, and flag feature impacts or WASM bundle changes; include screenshots or HTML snippets if output formats change.

## WASM Regex-Free Migration Plan
- Replace metadata, note, and scene-number regex usage with deterministic scanners; validate via `tests/parse_metadata_test.rs`, `tests/parse_notes_test.rs`, and `tests/parse_scene_heading_test.rs`.
- Reimplement boneyard and control-char stripping with a streaming filter that removes `/* */` blocks and known formatting code points; confirm with `tests/parse_action_test.rs` and `tests/parse_page_breaks_test.rs`.
- Rewrite `text_style_parser` as a stack-based tokenizer over `chars()`, preserving escapes and nesting; lean on its unit tests plus dialogue integrations in `tests/parse_dialogue_block_test.rs`.
- Gate the alternative parser behind a cargo feature such as `no-regex-parser`, enabling incremental rollout while running `cargo test`, `cargo test --all-features`, and WASM builds.
- After each major swap, rerun `cargo bench` and compare against `BENCHMARK_BASELINE.md` to ensure performance parity before flipping the default for WASM.
