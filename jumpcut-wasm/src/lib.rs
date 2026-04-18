use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_to_json_string(text: &str) -> String {
    let screenplay = jumpcut::parse(text);
    screenplay.to_json_string()
}

#[wasm_bindgen]
pub fn parse_to_fountain_string(text: &str) -> String {
    let screenplay = jumpcut::parse(text);
    screenplay.to_fountain()
}

#[cfg(feature = "fdx")]
#[wasm_bindgen]
pub fn parse_to_fdx_string(text: &str) -> String {
    let mut screenplay = jumpcut::parse(text);
    screenplay.to_final_draft()
}

#[cfg(feature = "fdx")]
#[wasm_bindgen]
pub fn parse_fdx_to_fountain_string(text: &str) -> Result<String, JsValue> {
    let screenplay =
        jumpcut::parse_fdx(text).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    Ok(screenplay.to_fountain())
}

#[cfg(feature = "fdx")]
#[wasm_bindgen]
pub fn parse_fdx_to_html_string(text: &str, include_head: bool) -> Result<String, JsValue> {
    let mut screenplay =
        jumpcut::parse_fdx(text).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    Ok(screenplay.to_html(include_head))
}

#[cfg(feature = "fdx")]
#[wasm_bindgen]
pub fn parse_fdx_to_pdf_bytes(text: &str) -> Result<Vec<u8>, JsValue> {
    let screenplay =
        jumpcut::parse_fdx(text).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    Ok(screenplay.to_pdf())
}

#[cfg(feature = "pdf")]
#[wasm_bindgen]
pub fn parse_to_pdf_bytes(text: &str) -> Vec<u8> {
    let screenplay = jumpcut::parse(text);
    screenplay.to_pdf()
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
    screenplay.to_html_with_options(jumpcut::rendering::html::HtmlRenderOptions {
        head: include_head,
        exact_wraps: exact_wraps || paginated,
        paginated,
        render_title_page: true,
        embed_courier_prime: false,
        embedded_courier_prime_css: None,
        ..Default::default()
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
    screenplay.to_html_with_options(jumpcut::rendering::html::HtmlRenderOptions {
        head: include_head,
        exact_wraps: exact_wraps || paginated,
        paginated,
        render_title_page: true,
        embed_courier_prime: false,
        embedded_courier_prime_css: Some(
            jumpcut::rendering::html::embedded_courier_prime_css_from_base64(
                regular_ttf_base64,
                italic_ttf_base64,
                bold_ttf_base64,
                bold_italic_ttf_base64,
            ),
        ),
        ..Default::default()
    })
}
