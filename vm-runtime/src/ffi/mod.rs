#[cfg(not(feature = "ffi"))]
compile_error!("\"ffi\" feature must be enabled for this module.");

mod ffi;
pub use ffi::*;
mod bare_ffi;
pub use bare_ffi::*;