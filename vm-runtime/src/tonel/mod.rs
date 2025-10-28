#[cfg(not(feature = "tonel"))]
compile_error!("\"tonel\" feature must be enabled for this module.");

mod loader;