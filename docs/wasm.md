# Using JumpCut from JavaScript

The `jumpcut-wasm` package lets browser and Node applications convert Fountain and FDX screenplays.

## Build the package

From the repository root, build a browser package:

```sh
./scripts/wasm/generate-package.sh
```

The files are written to `target/wasm-package/web-full`. Import the generated module and initialize it before converting a script:

```js
import init, { parse_to_html_string } from "./target/wasm-package/web-full/jumpcut_wasm.js";

await init();
const html = parse_to_html_string("INT. HOTEL - NIGHT\n\nAn empty lobby.", true);
```

For Node, build with `--target nodejs`:

```sh
./scripts/wasm/generate-package.sh --target nodejs
```

```js
const jumpcut = require("./target/wasm-package/node-full/jumpcut_wasm.js");
const html = jumpcut.parse_to_html_string("INT. HOTEL - NIGHT\n\nAn empty lobby.", true);
```

## Conversion functions

Fountain input can produce Fountain, JSON, HTML, FDX, or PDF:

- `parse_to_fountain_string(text)`
- `parse_to_json_string(text)`
- `parse_to_html_string(text, include_head)`
- `parse_to_html_string_with_options(text, include_head, exact_wraps, paginated)`
- `parse_to_fdx_string(text)`
- `parse_to_pdf_bytes(text)`

For FDX input, use `parse_fdx_to_fountain_string(text)`, `parse_fdx_to_html_string(text, include_head)`, or `parse_fdx_to_pdf_bytes(text)`.

## Optional features

The default build includes HTML, FDX, and PDF. These can be selected separately through Cargo features; JSON is always available.

PDF font subsetting shrinks PDFs by embedding only the glyphs a document uses. It makes PDFs smaller but adds a lot of code to the WASM package making it slower to load:

```sh
wasm-pack build --release --target web jumpcut-wasm --features pdf-font-subsetting
```

This feature also enables PDF output. The default WASM build compresses PDFs without subsetting fonts because the file sizes of most screenplays are tiny and I'd rather the WASM load faster. Native builds subset fonts by default.

## Embedded HTML fonts

For HTML with embedded fonts, supply the four font files as base64 strings. Your application must host or bundle these files separately. This example assumes `fountainText` contains your script:

```js
import init, {
  parse_to_html_string_with_embedded_courier_prime,
} from "./jumpcut_wasm.js";

await init();

const [regular, italic, bold, boldItalic] = await Promise.all([
  fetch("/fonts/CourierPrime-Regular.ttf").then(r => r.arrayBuffer()),
  fetch("/fonts/CourierPrime-Italic.ttf").then(r => r.arrayBuffer()),
  fetch("/fonts/CourierPrime-Bold.ttf").then(r => r.arrayBuffer()),
  fetch("/fonts/CourierPrime-BoldItalic.ttf").then(r => r.arrayBuffer()),
]);

const toBase64 = buf =>
  btoa(String.fromCharCode(...new Uint8Array(buf)));

const html = parse_to_html_string_with_embedded_courier_prime(
  fountainText,
  true,
  false,
  true,
  toBase64(regular),
  toBase64(italic),
  toBase64(bold),
  toBase64(boldItalic),
);
```
