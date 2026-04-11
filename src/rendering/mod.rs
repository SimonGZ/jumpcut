#[cfg(feature = "fdx")]
pub mod fdx;
#[cfg(feature = "html")]
pub mod html;
pub mod pdf;
#[cfg(any(feature = "fdx", feature = "html"))]
pub(crate) mod shared;
pub mod text;
