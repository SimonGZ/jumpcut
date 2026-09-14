# JumpCut

JumpCut converts screenplays between [Fountain](https://fountain.io/) and Final Draft’s FDX format. It also creates PDFs, HTML, plain text, and JSON.

Use it from the command line, in a Rust application, or from JavaScript through WebAssembly.

## Install

```sh
cargo install jumpcut
```

## Convert a screenplay

```sh
jumpcut script.fountain script.pdf
jumpcut script.fountain script.fdx
jumpcut script.fdx script.fountain
```

JumpCut chooses the output format from the filename. See the [command-line guide](docs/cli.md) for more examples and options.

## Formatting

JumpCut handles screenplay formatting and page breaks automatically. You can change the paper size, heading styles, and other formatting through your script’s metadata.

There are two "profiles" available that bundle together different pagination schemes. The default is called `jumpcut` and is very similar to industry standard programs like Final Draft, but with some small aesthetic tweaks (handles em-dashes better, handles cont'd and mores in dual dialogue more elegantly, avoids starting pages with parenthetical dialogue, etc). This profile reflects the author's tastes and is subject to change.

The other profile is called `industry` and produces nearly identical pagination to Final Draft, even in cases that I personally find unattractive like splitting halfway through an em-dash. 

Example:

```sh
jumpcut script.fountain script.pdf --render-profile industry
```

See the [formatting guide](docs/formatting-and-metadata.md) for details, or the [Fountain guide](docs/fountain-rules.md) for a syntax example.

## Use in an application

Add JumpCut to a Rust project:

```toml
jumpcut = { version = "2.1.0", default-features = false, features = ["lib-only"] }
```

Then parse and render a screenplay:

```rust
let mut screenplay = jumpcut::parse(&content);
let fdx = screenplay.to_fdx();
let html = screenplay.to_html(true);
```

See the [Rust guide](docs/rust.md) for HTML options, or the [JavaScript guide](docs/wasm.md) for browser and Node usage.

## License

MIT.
