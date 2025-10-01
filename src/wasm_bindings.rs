use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_to_json_string(text: &str) -> String {
    let screenplay = crate::parse(text);
    screenplay.to_json_string()
}

#[wasm_bindgen]
pub fn parse_to_fdx_string(text: &str) -> String {
    let mut screenplay = crate::parse(text);
    screenplay.to_final_draft()
}
