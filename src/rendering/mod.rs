#[cfg(feature = "fdx")]
pub mod fdx;
pub mod fountain;
#[cfg(feature = "html")]
pub mod html;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(any(feature = "fdx", feature = "html"))]
pub(crate) mod shared;
pub mod text;
