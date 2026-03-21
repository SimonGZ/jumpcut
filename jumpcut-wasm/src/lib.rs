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
