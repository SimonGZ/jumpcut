#[cfg(feature = "fdx")]
pub mod fdx;
#[cfg(feature = "html")]
pub mod html;
#[cfg(any(feature = "fdx", feature = "html"))]
pub(crate) mod shared;
pub mod pdf;
pub mod text;
