# Use JumpCut from JavaScript with WASM

JumpCut ships a small [`jumpcut-wasm`](../jumpcut-wasm) wrapper for browser and Node applications. It can parse Fountain or FDX and return Fountain, JSON, HTML, FDX, or PDF data.

This page covers the practical pieces:

- the JavaScript functions it exposes
- how to build a package for the web or Node
- which optional features affect bundle size
- the repository checks used when changing the wrapper

Embedded Courier Prime HTML export is documented separately in [`html-embedded-fonts.md`](html-embedded-fonts.md).

## Choose features

JumpCut uses Cargo features so you can choose how much functionality to compile into a WASM build.

Current wasm feature slices:

- JSON: always available
- HTML: optional
- FDX: optional
- PDF: optional
- PDF font subsetting: optional through `pdf-font-subsetting`

### PDF font subsetting and bundle size

Font subsetting embeds only the glyphs used by the current document. This can materially reduce generated PDFs, especially short documents where the four font faces account for most of the file, but the subsetting implementation also adds substantial code to the wasm module, so it's an opt-in choice:

```sh
wasm-pack build --release --target web jumpcut-wasm --features pdf-font-subsetting
```

The `pdf-font-subsetting` feature includes `pdf`; it does not need to be listed separately.

The default WASM package keeps lossless PDF stream compression but does not subset fonts. Native JumpCut builds subset fonts by default. With the repository's release profile and `wasm-opt -Os`, the tradeoff measured as follows:

| Build | Raw WASM | gzip | Brotli |
| --- | ---: | ---: | ---: |
| PDF before stream compression | 1,627,578 bytes | 667,190 bytes | 497,466 bytes |
| Default WASM with PDF compression | 1,655,073 bytes | 678,912 bytes | 506,150 bytes |
| PDF compression and font subsetting | 2,260,714 bytes | 901,770 bytes | 672,320 bytes |

Compression adds about 27 KB of raw WASM, while subsetting adds about 606 KB. The latter can meaningfully reduce generated PDFs, especially short documents where fonts account for most of the file, but most web applications should prefer the smaller default bundle.

## Functions available in JavaScript

The wasm wrapper currently exposes:

- `parse_to_fountain_string(text)`
- `parse_to_json_string(text)`
- `parse_to_html_string(text, include_head)`
- `parse_to_html_string_with_options(text, include_head, exact_wraps, paginated)`
- `parse_to_html_string_with_embedded_courier_prime(text, include_head, exact_wraps, paginated, regular_ttf_base64, italic_ttf_base64, bold_ttf_base64, bold_italic_ttf_base64)`
- `parse_to_fdx_string(text)`
- `parse_to_pdf_bytes(text)`
- `parse_fdx_to_fountain_string(text)`
- `parse_fdx_to_html_string(text, include_head)`
- `parse_fdx_to_pdf_bytes(text)`

## Build for the web or Node

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

## Use the generated package from Node

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

const fountain = jumpcut.parse_to_fountain_string(input);
const json = jumpcut.parse_to_json_string(input);
const html = jumpcut.parse_to_html_string(input, true);
const fdx = jumpcut.parse_to_fdx_string(input);

console.log(fountain.slice(0, 80));
console.log(json);
console.log(html.slice(0, 80));
console.log(fdx.slice(0, 80));
```

## Check a WASM change

The repo includes helper scripts for the wasm workflow:

- `./scripts/wasm/generate-package.sh`
  - builds the wasm wrapper
  - generates a web-targeted JS package by default
  - supports `--target nodejs` for a Node-targeted package
  - optionally runs a small smoke benchmark
- `./scripts/wasm/checks.sh`
  - repo-internal validation helper for wasm changes
  - runs tests and `wasm32` checks
  - generates a Node package, loads the compiled WASM, and validates every
    default-feature JS export against a small Fountain/FDX fixture
- `./scripts/wasm/report.sh`
  - repo-internal benchmark/size-report helper
  - emits bundle-size metrics, feature-slice metrics (`json_only`, `html_only`, `fdx_only`, `pdf_only`, `pdf_subset`), native guardrail metrics, and Node-side wasm runtime metrics

The current benchmark baseline artifact lives at:

```text
benchmarks/wasm/baseline-state.json
```
