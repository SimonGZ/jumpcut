#[cfg(not(target_arch = "wasm32"))]
pub mod diagnostics;
pub mod fdx;
pub mod model;
pub mod pagination;
pub mod parser;
pub mod rendering;
mod text_style_parser;

pub use fdx::parse_fdx;
pub use model::*;
pub use parser::parse;
