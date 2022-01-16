#[cfg(feature = "ffi")]
mod ffi_feature;
#[cfg(feature = "threaded_ffi")]
mod threaded_ffi_feature;

#[cfg(feature = "ffi")]
pub use ffi_feature::ffi_feature;
#[cfg(feature = "threaded_ffi")]
pub use threaded_ffi_feature::threaded_ffi_feature;
