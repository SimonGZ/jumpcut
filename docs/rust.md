# Using JumpCut from Rust

The [README](../README.md#use-in-an-application) shows how to add JumpCut and convert a screenplay.

## HTML options

Use `to_html_with_options` when you need control over the HTML output. This example includes Courier Prime in the file:

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

Without embedded fonts, the HTML uses installed fonts, falling back to Courier. Embedding fonts makes the file larger.
