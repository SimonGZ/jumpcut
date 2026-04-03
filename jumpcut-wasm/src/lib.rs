use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_to_json_string(text: &str) -> String {
    let screenplay = jumpcut::parse(text);
    screenplay.to_json_string()
}

#[cfg(feature = "fdx")]
#[wasm_bindgen]
pub fn parse_to_fdx_string(text: &str) -> String {
    let mut screenplay = jumpcut::parse(text);
    screenplay.to_final_draft()
}

#[cfg(feature = "html")]
#[wasm_bindgen]
pub fn parse_to_html_string(text: &str, include_head: bool) -> String {
    let mut screenplay = jumpcut::parse(text);
    screenplay.to_html(include_head)
}

#[cfg(feature = "html")]
#[wasm_bindgen]
pub fn parse_to_html_string_with_options(
    text: &str,
    include_head: bool,
    exact_wraps: bool,
    paginated: bool,
) -> String {
    let mut screenplay = jumpcut::parse(text);
    screenplay.to_html_with_options(jumpcut::html_output::HtmlRenderOptions {
        head: include_head,
        exact_wraps: exact_wraps || paginated,
        paginated,
        embed_courier_prime: false,
        embedded_courier_prime_css: None,
    })
}

#[cfg(feature = "html")]
#[wasm_bindgen]
pub fn parse_to_html_string_with_embedded_courier_prime(
    text: &str,
    include_head: bool,
    exact_wraps: bool,
    paginated: bool,
    regular_ttf_base64: &str,
    italic_ttf_base64: &str,
    bold_ttf_base64: &str,
    bold_italic_ttf_base64: &str,
) -> String {
    let mut screenplay = jumpcut::parse(text);
    screenplay.to_html_with_options(jumpcut::html_output::HtmlRenderOptions {
        head: include_head,
        exact_wraps: exact_wraps || paginated,
        paginated,
        embed_courier_prime: false,
        embedded_courier_prime_css: Some(
            jumpcut::html_output::embedded_courier_prime_css_from_base64(
                regular_ttf_base64,
                italic_ttf_base64,
                bold_ttf_base64,
                bold_italic_ttf_base64,
            ),
        ),
    })
}
