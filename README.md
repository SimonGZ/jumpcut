# JumpCut

JumpCut is a Rust utility designed to convert the [Fountain screenwriting markup format][fountain] into [Final Draft FDX files][FDX] (the industry standard in Hollywood) or into HTML.

JumpCut can be used as a command-line utility, a Rust library, or as a WASM package. Because of this, the project utilizes cargo [features][] so that different parts like the command-line utility can be turned off to save binary size.

## Installation

If you want to use JumpCut as a command-line utility, you can install it via Cargo.

```sh
cargo install jumpcut
```

To use JumpCut as a library, you can specify the following in your Cargo.toml so that the command-line features are not added to your project:

`jumpcut = { version = "0.7", default-features = false, features = ["lib-only"] }`

## Usage

Once installed, you can pass JumpCut a text file and it will parse it and output it as either an FDX, HTML, or JSON. The full options from the help text are listed below.

```
USAGE:
    jumpcut [OPTIONS] <input> [output]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    Formats (FDX, HTML, JSON) [default: fdx]

ARGS:
    <input>     Input file, pass a dash ("-") to receive stdin
    <output>    Output file, stdout if not present
```

To use JumpCut within a Rust program, you can examine the [main.rs](src/bin/main.rs) file for an example of calling the library, but the basics are depicted below:

```rust
let mut screenplay: Screenplay = parse(&content); // content is a String provided by your application
let output_fdx: String = screenplay.to_final_draft();
let output_html: String = screenplay.to_html();
```

## Development Plans

I have no current plans to expand this project. I've used it internally for a few years and it meets my current needs (converting my own screenplays locally and powering my website [FountainLoader.com][]).

I have open-sourced it in case it can be useful to other developers and screenwriters.

## Changelog

- 0.7.1: Improving documentation.
- 0.7.0: Initial public release. Supports FDX, HTML, and JSON output.

## License

JumpCut is licensed under the terms of the MIT license.

[fountain]: https://fountain.io/
[FDX]: https://www.finaldraft.com/
[features]: https://doc.rust-lang.org/cargo/reference/features.html
[FountainLoader.com]: https://fountainloader.com
