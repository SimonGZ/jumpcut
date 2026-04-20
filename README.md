# JumpCut

JumpCut is a Rust utility designed to convert [Fountain screenwriting markup format][fountain] and Final Draft documents (FDX) into Fountain, FDX, HTML, JSON, text, and PDF formats.

It was created by a working screenwriter to match the industry-standard conventions for Hollywood screenplays (lines per page, margins, dialogue splits, etc).

JumpCut can be used as a command-line utility, a Rust library, or as a WASM package.

## Installation

If you want to use JumpCut as a command-line utility, you can install it via Cargo.

```sh
cargo install jumpcut
```

To use JumpCut as a library, you can specify the following in your Cargo.toml so that the command-line features are not added to your project:

`jumpcut = { version = "1.0.0-beta.1", default-features = false, features = ["lib-only"] }`

## Usage

Once installed, you can pass JumpCut a text file and it will parse it and output it as FDX, HTML, JSON, text, or PDF. The full options from the help text are listed below.

```
A tool for converting Fountain and Final Draft screenplay documents into Fountain, FDX, HTML, JSON, text, and optional PDF formats.

Usage: jumpcut [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Input file, pass a dash ("-") to receive stdin
  [OUTPUT]  Output file in the legacy positional form

Options:
  -f, --format <FORMAT>
          Formats (Fountain, FDX, HTML, JSON, text, PDF)
      --paginate
          Render text output with pagination
      --exact-wraps
          Render HTML output with exact Final Draft-style wraps
      --embed-courier-prime
          Embed Courier Prime font files directly into HTML CSS
      --line-numbers
          Show line numbers in text output
      --render-profile <RENDER_PROFILE>
          Override the layout/render profile instead of using fmt metadata [possible values: industry, balanced]
      --no-continueds
          Suppress (CONT'D)/(MORE) style continued markers in render outputs
      --no-title-page
          Suppress title-page output for HTML and PDF renders
  -o, --output <FILE>
          Output file
  -w, --write
          Auto-derive an output file path from the input stem and format
  -m, --metadata [<FILE>]
          Optional Fountain file to merge as metadata. Defaults to "metadata.fountain" if flag is present without a value
  -h, --help
          Print help
  -V, --version
          Print version
```

Examples:

```sh
jumpcut script.fountain script.fdx

# Explicit output flag
jumpcut script.fountain -o script.fdx

# Auto-derive the output path from the input
jumpcut script.fountain -w
jumpcut script.fountain -w -f pdf   # writes script.pdf
```

`-w` is the explicit "write next to the source" mode. `-o` always expects a file path.

To use JumpCut within a Rust program, look at [main.rs](src/bin/main.rs) file for an example of calling the library, but the basics are...

```rust
let mut screenplay: Screenplay = parse(&content); // content is a String of fountain text
let output_fdx: String = screenplay.to_final_draft();
let output_html: String = screenplay.to_html();
```

## Formatting and Metadata

You can customize JumpCut's output.

There are two built-in profiles that act like presets. They bundle together different pagination and output settings:

- `industry`: the default. This aims for the kind of screenplay pagination and continuation behavior used by major industry tools (like Final Draft).
- `balanced`: a more opinionated profile that aims for cleaner-looking page breaks, dash wrapping, and `(MORE)` / `(CONT'D)` choices. NOTE: This profile is subject to changes based on the changing opinions of the software's author.

You can set those presets (called render-profiles by the app) and other frequent customizations with CLI flags like `--render-profile`, `--no-continueds`, and `--no-title-page`

More specific formatting, margin, and pagination tweaks can be set in a `fmt` string in the metadata section at the top of a Fountain document.

JumpCut also supports per-element layout overrides (aka "cheats"). These modify an individual paragraph to change the margins or spacing, which can be useful when trying to pull up a line and fit more on a page. In Fountain, those can be written as modifier notes like `[[ .lift-1 ]]` and `[[ .widen-2 ]]`, and equivalent paragraph-level spacing / width deviations are preserved during FDX import and export.

If you want the full reference for `fmt`, profile overrides, and `--metadata` / `-m`, see [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).

## WASM

JumpCut also ships an in-repo wasm wrapper crate at [jumpcut-wasm](jumpcut-wasm), so that JumpCut can be used in websites.

For the wasm wrapper API, Cargo feature model, package-generation workflow, and internal size/report tooling, see [`docs/wasm.md`](docs/wasm.md).

Embedded Courier Prime HTML export is documented in [`docs/html-embedded-fonts.md`](docs/html-embedded-fonts.md).

## Diagnostics

Pagination diagnostics and PDF parity tooling are documented in [`docs/diagnostics.md`](docs/diagnostics.md).

## Development Plans

I have open-sourced this project in case it can be useful to other developers and screenwriters. But I mostly develop it for my own use on my own projects. Features are added as-needed for my workflow.

## License

JumpCut is licensed under the terms of the MIT license.

[fountain]: https://fountain.io/
[FDX]: https://www.finaldraft.com/
[features]: https://doc.rust-lang.org/cargo/reference/features.html
[FountainLoader.com]: https://fountainloader.com
