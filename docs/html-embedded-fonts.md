# Embedded HTML Fonts

JumpCut can optionally embed the four Courier Prime TTF files directly into generated HTML as `data:` URLs inside CSS `@font-face` rules.

This is useful when you want a single portable HTML file that does not depend on local font installation.

## Native CLI and Rust Library

The embedded-font path is off by default.

CLI:

```sh
jumpcut --format html --embed-courier-prime input.fountain output.html
```

Rust:

```rust
let mut screenplay = jumpcut::parse(&content);
let html = screenplay.to_html_with_options(jumpcut::html_output::HtmlRenderOptions {
    head: true,
    exact_wraps: false,
    paginated: true,
    embed_courier_prime: true,
    embedded_courier_prime_css: None,
});
```

When `embed_courier_prime` is `false`, JumpCut keeps the normal local-font lookup plus Courier fallbacks.

## WASM

For wasm, JumpCut does **not** compile the Courier Prime font bytes into the `.wasm` binary.

Instead, the wasm wrapper exposes a runtime-supplied path:

- `parse_to_html_string(text, include_head)`
- `parse_to_html_string_with_options(text, include_head, exact_wraps, paginated)`
- `parse_to_html_string_with_embedded_courier_prime(text, include_head, exact_wraps, paginated, regular_ttf_base64, italic_ttf_base64, bold_ttf_base64, bold_italic_ttf_base64)`

That lets the app fetch or bundle the font files separately and only pay the cost when the user actually requests embedded-font export.

Example:

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

## Size Notes

Embedding Courier Prime materially increases HTML size because the font data is stored inline.

That is intentional for the standalone export case, but it should remain opt-in.
