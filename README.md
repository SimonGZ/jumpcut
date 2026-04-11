# JumpCut

JumpCut is a Rust utility designed to convert the [Fountain screenwriting markup format][fountain] into [Final Draft FDX files][FDX] (the industry standard in Hollywood), HTML, JSON, text, and PDF.

JumpCut can be used as a command-line utility, a Rust library, or as a WASM package. Because of this, the project utilizes cargo [features][] so that different parts like the command-line utility can be turned off to save binary size.

Embedded Courier Prime HTML export is documented in [docs/html-embedded-fonts.md](docs/html-embedded-fonts.md).

## Installation

If you want to use JumpCut as a command-line utility, you can install it via Cargo.

```sh
cargo install jumpcut
```

To use JumpCut as a library, you can specify the following in your Cargo.toml so that the command-line features are not added to your project:

`jumpcut = { version = "1.0.0-beta", default-features = false, features = ["lib-only"] }`

## WASM Package

JumpCut also ships an in-repo wasm wrapper crate at [jumpcut-wasm](jumpcut-wasm).

That wrapper exposes three JS-facing functions:

- `parse_to_json_string(text)`
- `parse_to_html_string(text, include_head)`
- `parse_to_html_string_with_options(text, include_head, exact_wraps, paginated)`
- `parse_to_html_string_with_embedded_courier_prime(text, include_head, exact_wraps, paginated, regular_ttf_base64, italic_ttf_base64, bold_ttf_base64, bold_italic_ttf_base64)`
- `parse_to_fdx_string(text)`

### Build The WASM Wrapper

The low-level Rust build is:

```sh
cargo build -p jumpcut-wasm --target wasm32-unknown-unknown --release
```

To generate a Node-compatible JS package from the compiled `.wasm`, use:

```sh
./autoresearch-wasm.node.sh --smoke
```

That script will:

- build `jumpcut-wasm`
- ensure `wasm-bindgen-cli` is available
- generate a Node-targeted package under `target/autoresearch-wasm/node-full`
- run a small smoke benchmark

If you want the generated package without the smoke shortcut, run:

```sh
./autoresearch-wasm.node.sh
```

### Use The Generated Package From Node

After running `./autoresearch-wasm.node.sh`, the generated package lives under:

```text
target/autoresearch-wasm/node-full
```

Example:

```js
const jumpcut = require("./target/autoresearch-wasm/node-full/jumpcut_wasm.js");

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

### WASM Checks And Benchmarks

The repo includes helper scripts for the wasm workflow:

- `./autoresearch-wasm.checks.sh`
  - runs tests
  - checks `jumpcut-wasm` for `wasm32-unknown-unknown`
  - runs the Node-side smoke path
- `./autoresearch-wasm.sh`
  - emits full bundle size metrics
  - emits `json_only` / `html_only` / `fdx_only` size metrics
  - emits native parser guardrail metrics
  - emits Node-side wasm runtime metrics

Those scripts are what the repo currently uses to validate wasm changes.

## Usage

Once installed, you can pass JumpCut a text file and it will parse it and output it as FDX, HTML, JSON, text, or PDF. The full options from the help text are listed below.

```
USAGE:
    jumpcut [OPTIONS] <input> [output]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    Formats (FDX, HTML, JSON, text, PDF) [default: fdx]
    -o, --output <FILE>      Output file.
    -w, --write              Auto-derive an output path from the input stem and format.
    -m, --metadata <FILE>    Optional Fountain file to prepend as metadata. Defaults to "metadata.fountain" if flag is present without a value.
        --paginate           Render paginated text or exact-wrap paginated HTML
        --line-numbers       Show line numbers in text output
        --exact-wraps        Render HTML with exact Final Draft-style wraps
        --render-profile <render-profile>
                             Override metadata-driven render profile [possible values: final-draft, balanced]
        --no-continueds      Suppress (CONT'D)/(MORE) style continued markers in text, HTML, or PDF output

ARGS:
    <input>     Input file, pass a dash ("-") to receive stdin
    <output>    Output file, stdout if not present
```

Output path forms:

```sh
# Legacy positional output path
jumpcut script.fountain script.fdx

# Explicit output flag
jumpcut script.fountain -o script.fdx

# Auto-derive the output path from the input stem and format
jumpcut script.fountain -w
jumpcut script.fountain -w -f pdf   # writes script.pdf
```

`-w` is the explicit "write next to the source" mode. `-o` always expects a file path.

To use JumpCut within a Rust program, you can examine the [main.rs](src/bin/main.rs) file for an example of calling the library, but the basics are depicted below:

```rust
let mut screenplay: Screenplay = parse(&content); // content is a String provided by your application
let output_fdx: String = screenplay.to_final_draft();
let output_html: String = screenplay.to_html();
```

### CLI Render Profile Override

Formatting and metadata details now live in [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md), including:

- `--render-profile`
- `--no-continueds`
- `fmt` metadata tokens
- `--metadata` / `-m`

## Formatting Metadata (`fmt`)

`fmt` metadata controls shared layout and rendering behavior across pagination and multiple output formats.

For the full token reference and examples, see [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).

## Prepending Metadata

JumpCut can prepend metadata from a separate Fountain file via `--metadata` / `-m`.

For default-file lookup rules and runnable examples, see [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).

## Pagination Diagnostics

The pagination/parity harness includes several ignored tests that generate review packets and debug artifacts under `target/pagination-debug/`.

If you want a single command that rebuilds all of those diagnostics, install [`just`](https://github.com/casey/just):

```sh
cargo install just
```

Then run:

```sh
just pagination-diagnostics
```

That recipe calls the dedicated Rust diagnostics tool:

```sh
cargo run --bin pagination-diagnostics -- all
```

The tool currently regenerates:

- Big Fish review packet
- Big Fish full-script page-break review packet
- Big Fish line-break parity packet
- Little Women windowed review packet
- Little Women full-script page-break review packet
- Little Women line-break parity packet
- Mostly Genius full-script page-break review packet

## PDF Parity Checks

The repo also includes a PDF word-position parity checker:

```sh
python3 tools/check_corpus_pdf_parity.py
```

That runs the wired corpus PDF checks and writes reports under:

```text
target/pdf-placement-diagnostics/
```

If you want to compare an ad hoc script without adding it to the permanent corpus, use one or more `--case` entries:

```sh
python3 tools/check_corpus_pdf_parity.py \
  --no-default-cases \
  --case my-script /abs/path/to/script.fountain /abs/path/to/reference.pdf
```

You can provide `--case` more than once in the same run. The script generates a PDF with JumpCut, compares it against the supplied reference PDF using `pdftotext -bbox-layout`, and writes a report packet for each case.

If a reference differs only in letter case, you can opt into case-insensitive text matching before geometry comparison:

```sh
python3 tools/check_corpus_pdf_parity.py \
  --no-default-cases \
  --ignore-case \
  --case mostly-genius /abs/path/to/mostly-genius.fountain /abs/path/to/mostly-genius.pdf
```

Local dependency:

```sh
sudo apt install poppler-utils
```

That provides `pdftotext`, which the parity checker requires.
- Mostly Genius line-break parity packet
- the extra paginated-output JSON dumps and visual-comparison export used for manual debugging

There are also narrower convenience tasks:

```sh
just big-fish-diagnostics
just mostly-genius-diagnostics
```

## Development Plans

I have open-sourced this project in case it can be useful to other developers and screenwriters. But I mostly develop it for my own use on my own projects. Features are added as-needed for my workflow.

## License

JumpCut is licensed under the terms of the MIT license.

[fountain]: https://fountain.io/
[FDX]: https://www.finaldraft.com/
[features]: https://doc.rust-lang.org/cargo/reference/features.html
[FountainLoader.com]: https://fountainloader.com
