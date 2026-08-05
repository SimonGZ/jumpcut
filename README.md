# JumpCut

JumpCut is a Rust utility designed to convert [Fountain screenwriting markup format][fountain] and FDX documents into Fountain, FDX, HTML, JSON, text, and PDF formats.

It was created by a working screenwriter to match the industry-standard conventions for Hollywood screenplays (lines per page, margins, dialogue splits, etc).

JumpCut can be used as a command-line utility, a Rust library, or as a WASM package.

## Installation

If you want to use JumpCut as a command-line utility, you can install it via Cargo.

```sh
cargo install jumpcut
```

To use JumpCut as a library, you can specify the following in your Cargo.toml so that the command-line features are not added to your project:

`jumpcut = { version = "1.1.0-beta.1", default-features = false, features = ["lib-only"] }`

## Usage

Once installed, you can pass JumpCut a text file and it will parse it and output it as FDX, HTML, JSON, text, or PDF. The full options from the help text are listed below.

```
A tool for converting Fountain and FDX screenplay documents into Fountain, FDX, HTML, JSON, text, and optional PDF formats.

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
          Render HTML output with exact screenplay wraps
      --embed-courier-prime
          Embed bundled Courier Prime, or the family from --font-dir, directly into HTML CSS
      --line-numbers
          Show line numbers in text output
      --render-profile <RENDER_PROFILE>
          Override the layout/render profile [possible values: jumpcut, industry]
      --no-continueds
          Suppress (CONT'D)/(MORE) markers in render outputs
      --no-title-page
          Suppress title-page output for HTML and PDF renders
      --font-dir <DIR>
          Provide custom font family containing regular.ttf, bold.ttf, italic.ttf, and bold-italic.ttf (Only for PDF/HTML; must be used with --embed-courier-prime)
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
jumpcut script.fountain -w -f pdf   # writes script.pdf in same directory
```

### PDF bookmarks

PDF output includes a document outline built from screenplay scene headings.

### Custom PDF and HTML fonts

By default, PDF output uses an embedded Courier Prime family. To use a custom family (for example, one that covers Cyrillic characters) pass `--font-dir` with a directory containing all four TrueType faces.

```text
my-fonts/
├── regular.ttf
├── bold.ttf
├── italic.ttf
└── bold-italic.ttf
```

The following Courier Prime-style aliases are also accepted, one per role:

```text
courier-prime.ttf
courier-prime-bold.ttf
courier-prime-italic.ttf
courier-prime-bold-italic.ttf
```

Only TrueType-outline `.ttf` files are supported.

For HTML output, combine `--font-dir` with `--embed-courier-prime`; the custom faces are embedded as `data:` URLs in place of the bundled Courier Prime files.

```sh
jumpcut screenplay.fountain --format pdf --output screenplay.pdf --font-dir ./my-fonts
jumpcut screenplay.fountain --format html --output screenplay.html --embed-courier-prime --font-dir ./my-fonts
```

## Library Usage

To use JumpCut within a Rust program, look at [main.rs](src/bin/main.rs) for an example of calling the library, but the basics are...

```rust
let mut screenplay: Screenplay = parse(&content); // content is a String of fountain text
let output_fdx: String = screenplay.to_fdx();
let output_html: String = screenplay.to_html();
```

## Formatting and Metadata

You can customize JumpCut's output.

There are two canonical profiles that bundle together pagination and output settings:

- `jumpcut`: the default, opinionated profile. It favors cleaner dash wrapping and page breaks, avoids beginning a continuation page with a parenthetical when another legal split exists, and uses less intrusive continuation behavior.
- `industry`: the stable compatibility profile. It follows conventional screenplay pagination behavior using an ordered correction model and honors imported FDX sentence-correction and `PaginateAs` settings.

For migration, `balanced` is accepted as an alias for `jumpcut`, and `final-draft` is accepted as an alias for `industry`. Canonical output and new examples use only `jumpcut` and `industry`.

You can set these presets (called render-profiles by the app) and other frequent customizations with CLI flags like `--render-profile`, `--no-continueds`, and `--no-title-page`

More specific formatting, margin, and pagination tweaks can be set in a `fmt` string in the metadata section at the top of a Fountain document.

JumpCut also supports per-element layout overrides (aka "cheats"). These modify an individual paragraph to change the margins or spacing, which can be useful when trying to pull up a line and fit more on a page. In Fountain, those can be written as modifier notes like `[[ .lift-1 ]]` and `[[ .widen-2 ]]`. These paragraph-level spacing / width tweaks are preserved during FDX import and export.

If you want the full reference for `fmt`, profile overrides, and `--metadata` / `-m`, see [`docs/formatting-and-metadata.md`](docs/formatting-and-metadata.md).

## WASM

JumpCut also ships an in-repo wasm wrapper crate at [jumpcut-wasm](jumpcut-wasm), so that JumpCut can be used in websites.

For the wasm wrapper API, Cargo feature model, package-generation workflow, and internal size/report tooling, see [`docs/wasm.md`](docs/wasm.md).

The default wasm PDF renderer uses lossless stream compression without font subsetting to keep the downloaded module smaller. Native builds subset fonts by default; wasm consumers can opt into the same smaller generated PDFs with the `pdf-font-subsetting` feature. The measured bundle and PDF size tradeoffs are documented in [`docs/wasm.md`](docs/wasm.md#pdf-font-subsetting-and-bundle-size).

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
