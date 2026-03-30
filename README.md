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

## WASM Package

JumpCut also ships an in-repo wasm wrapper crate at [jumpcut-wasm](jumpcut-wasm).

That wrapper exposes three JS-facing functions:

- `parse_to_json_string(text)`
- `parse_to_html_string(text, include_head)`
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

Once installed, you can pass JumpCut a text file and it will parse it and output it as either an FDX, HTML, or JSON. The full options from the help text are listed below.

```
USAGE:
    jumpcut [OPTIONS] <input> [output]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    Formats (FDX, HTML, JSON) [default: fdx]
    -m, --metadata <FILE>    Optional Fountain file to prepend as metadata. Defaults to "metadata.fountain" if flag is present without a value.

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

## Custom Formatting for Final Draft (FDX) Export

When converting your screenplay to **Final Draft (FDX)** format, you can specify custom formatting options using the `fmt` metadata key. This allows you to control various aspects of the FDX output, such as text styles, spacing, and margins.

To use these options, add a `fmt` key to your screenplay's metadata (optional `key: value` statements placed at the top of a document), followed by a space-separated list of options.

JumpCut treats `fmt` options in two layers:

- template options such as `multicam` establish a base layout/style profile
- explicit geometry knobs such as `ssbsh`, `dsd`, `dl-*`, and `dr-*` then override that base, regardless of where they appear in the `fmt` string

**Example:**

```
Title: My Awesome Screenplay
Author: John Doe
Fmt: bsh ush acat dsd dl-1.5 dr-7.0
```

### Available `fmt` Options

  * **`multicam`**: **Multicam Base Template**. Applies JumpCut's shared multicam layout profile as a starting point for pagination/FDX formatting.
  * **`bsh`**: **Bold Scene Headings**. Makes all scene headings bold.
  * **`ush`**: **Underlined Scene Headings**. Underlines all scene headings.
      * Note: `bsh` and `ush` can be combined (e.g., `bsh ush` for bold and underlined scene headings).
  * **`acat`**: **All Caps Action Text**. Converts all action text to uppercase.
  * **`ssbsh`**: **Single Space Before Scene Headings**. Reduces the space before scene headings from the default (24 points) to 12 points.
  * **`dsd`**: **Double-Spaced Dialogue**. Changes dialogue spacing from single to double.
  * **`cfd`**: **Courier Final Draft Font**. Uses "Courier Final Draft" as the primary font instead of the default "Courier Prime".
  * **`dl-X.XX`**: **Custom Dialogue Left Indent**. Sets the left indent for dialogue blocks. Replace `X.XX` with a numerical value (e.g., `dl-1.25`). The default is 2.50 inches.
  * **`dr-X.XX`**: **Custom Dialogue Right Indent**. Sets the right indent for dialogue blocks. Replace `X.XX` with a numerical value (e.g., `dr-6.00`). The default is 6.00 inches.

### Combined Example

To start from the multicam template, keep its double-spaced dialogue, and then override the dialogue margins explicitly:

```
Fmt: multicam bsh ush acat dsd dl-2.0 dr-5.5
```

## Prepending Metadata

JumpCut allows you to prepend content from a separate Fountain file as metadata to your main screenplay. This is useful for managing common metadata (like title, author, copyright, fmt) across multiple screenplay files without duplicating it in each one.

You can use the `--metadata` (or `-m`) option to specify a metadata file.

### Usage

To use this feature, add the `--metadata` flag to your command.

```
jumpcut <screenplay-file> --metadata <metadata-file>
jumpcut <screenplay-file> -m <metadata-file>
```

If you provide the `--metadata` flag without a file path, JumpCut will look for a file named `metadata.fountain`. The location of this default file depends on your input:

*   **If your input is a file:** JumpCut will look for `metadata.fountain` in the same directory as your input screenplay.
*   **If your input is from stdin (`-`):** JumpCut will look for `metadata.fountain` in the current working directory.

### Examples

*   **Using a default metadata file alongside an input file:**
    ```sh
    jumpcut -m my_screenplay.fountain -f fdx > my_screenplay.fdx
    # Looks for 'metadata.fountain' in the same directory as 'my_screenplay.fountain'
    ```

*   **Using a default metadata file with stdin input:**
    ```sh
    cat my_screenplay.fountain | jumpcut -m -f html > my_screenplay.html
    # Looks for 'metadata.fountain' in the current working directory
    ```

*   **Specifying a custom metadata file:**
    ```sh
    jumpcut -m ~/my_templates/common_header.fountain my_screenplay.fountain -f json > my_screenplay.json
    # Uses 'common_header.fountain' from your templates directory
    ```

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
