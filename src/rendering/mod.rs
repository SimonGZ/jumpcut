#[cfg(feature = "fdx")]
pub(crate) mod fdx;
#[cfg(feature = "html")]
pub(crate) mod html;
#[cfg(any(feature = "fdx", feature = "html"))]
pub(crate) mod shared;
