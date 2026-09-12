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

/// PDF body glyphs advance on this grid, regardless of the font's natural width.
#[cfg(any(feature = "html", feature = "pdf"))]
pub(crate) const BODY_TEXT_CELL_WIDTH: f32 = 7.0;
