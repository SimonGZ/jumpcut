# WASM

JumpCut ships an in-repo wasm wrapper crate at [`jumpcut-wasm`](../jumpcut-wasm).

This document covers:

- what the wasm wrapper exposes
- how Cargo features affect wasm size
- how to build a Node-compatible package
- the internal wasm checks and size-report workflow

Embedded Courier Prime HTML export is documented separately in [`html-embedded-fonts.md`](html-embedded-fonts.md).

## Feature Model

JumpCut uses Cargo features so that consumers can choose how much functionality to compile into a wasm build.

Current wasm feature slices:

- JSON: always available
- HTML: optional
- FDX: optional
- PDF: optional

The default wasm build includes HTML, FDX, and PDF.

That means you can keep the default "everything on" build for a full web app, or build narrower packages when file size matters more than format coverage.

## JS-Facing Exports

The wasm wrapper currently exposes:

- `parse_to_json_string(text)`
- `parse_to_html_string(text, include_head)`
- `parse_to_html_string_with_options(text, include_head, exact_wraps, paginated)`
- `parse_to_html_string_with_embedded_courier_prime(text, include_head, exact_wraps, paginated, regular_ttf_base64, italic_ttf_base64, bold_ttf_base64, bold_italic_ttf_base64)`
- `parse_to_fdx_string(text)`
- `parse_to_pdf_bytes(text)`

## Build The WASM Wrapper

The low-level Rust build is:

```sh
cargo build -p jumpcut-wasm --target wasm32-unknown-unknown --release
```

To generate a browser/web-compatible JS package from the compiled `.wasm`, use:

```sh
./scripts/wasm/generate-package.sh --smoke
```

That script will:

- build `jumpcut-wasm`
- ensure `wasm-bindgen-cli` is available
- generate a web-targeted package under `target/wasm-package/web-full`
- run a small smoke benchmark

If you want a Node-targeted package instead, run:

```sh
./scripts/wasm/generate-package.sh --target nodejs --smoke
```

If you want the generated package without the smoke shortcut, run:

```sh
./scripts/wasm/generate-package.sh
```

## Use The Generated Package From Node

After running `./scripts/wasm/generate-package.sh`, the generated package lives under:

```text
target/wasm-package/web-full
```

For a Node-targeted package, use:

```js
const jumpcut = require("./target/wasm-package/node-full/jumpcut_wasm.js");

const input = `Title: Example

INT. HOUSE - DAY

Hello, world.`;

const json = jumpcut.parse_to_json_string(input);
const html = jumpcut.parse_to_html_string(input, true);
const fdx = jumpcut.parse_to_fdx_string(input);

console.log(json);
console.log(html.slice(0, 80));
console.log(fdx.slice(0, 80));
```

## Checks And Reports

The repo includes helper scripts for the wasm workflow:

- `./scripts/wasm/generate-package.sh`
  - builds the wasm wrapper
  - generates a web-targeted JS package by default
  - supports `--target nodejs` for a Node-targeted package
  - optionally runs a small smoke benchmark
- `./scripts/wasm/checks.sh`
  - repo-internal validation helper for wasm changes
  - runs tests and `wasm32` checks
  - runs the smoke package-generation path
- `./scripts/wasm/report.sh`
  - repo-internal benchmark/size-report helper
  - emits bundle-size metrics, feature-slice metrics (`json_only`, `html_only`, `fdx_only`, `pdf_only`), native guardrail metrics, and Node-side wasm runtime metrics

The current benchmark baseline artifact lives at:

```text
benchmarks/wasm/baseline-state.json
```
