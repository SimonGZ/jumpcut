#[cfg(feature = "no-regex-parser")]
mod new_impl;

#[cfg(not(feature = "no-regex-parser"))]
mod legacy_impl;

#[cfg(feature = "no-regex-parser")]
#[allow(unused_imports)]
pub use new_impl::*;

#[cfg(not(feature = "no-regex-parser"))]
#[allow(unused_imports)]
pub use legacy_impl::*;
