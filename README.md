# JumpCut

JumpCut converts screenplay files between [Fountain][fountain] and [FDX][fdx], and can render them as Fountain, FDX, HTML, JSON, plain text, or PDF.

It is available as a command-line tool, a Rust library, and a WASM package. It is built around the spacing, wrapping, and pagination details that make a screenplay look like a screenplay.

## Start here

### Install the command-line tool

```sh
cargo install jumpcut
```

### Convert a screenplay

```sh
# Convert a Fountain file to FDX
jumpcut script.fountain script.fdx

# Create a PDF with the same filename as the input
jumpcut script.fountain --format pdf --write

# Read from stdin and write HTML to stdout
cat script.fountain | jumpcut - --format html
```

The input may be a `.fountain` or `.fdx` file. If you don't specify an output filename or add `--write`, the result is written to stdout. Run `jumpcut --help` for the short command reference, or see the [CLI guide](docs/cli.md) for examples and all available options.

## Documentation

### Using JumpCut

- [Convert files from the command line](docs/cli.md)
- [Adjust formatting, pagination, and metadata](docs/formatting-and-metadata.md)
- [Check Fountain syntax supported by JumpCut](docs/fountain-rules.md)
- [Make a self-contained HTML file with embedded fonts](docs/html-embedded-fonts.md)
- [Use JumpCut from JavaScript with WASM](docs/wasm.md)

### Working on JumpCut

- [Understand the pagination model](docs/pagination-spec.md)
- [Understand dialogue split decisions](docs/dialogue-split-analysis.md)
- [See the Industry compatibility rules](docs/industry-dialogue-splitting-rules.md)
- [Run pagination and PDF diagnostics](docs/diagnostics.md)

## Rust library

Add JumpCut without its command-line interface:

```toml
jumpcut = { version = "1.1.0-beta.1", default-features = false, features = ["lib-only"] }
```

Then parse and render a screenplay:

```rust
let mut screenplay = jumpcut::parse(&content);
let fdx = screenplay.to_fdx();
let html = screenplay.to_html();
```

The library exposes the same parsing and rendering pipeline used by the CLI.

## Pagination profiles

JumpCut has two built-in pagination profiles:

- `jumpcut` is the default. It favors clean wraps and balanced continuation pages.
- `industry` is the compatibility option for conventional screenplay pagination and imported FDX behavior.

Choose a profile with `--render-profile` or the `fmt` metadata key. The [formatting guide](docs/formatting-and-metadata.md) explains the differences and the available fine-tuning options.

## License

JumpCut is released under the MIT license.

[fountain]: https://fountain.io/
[fdx]: https://www.finaldraft.com/
