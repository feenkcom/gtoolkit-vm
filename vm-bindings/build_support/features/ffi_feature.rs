#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{CompilationUnit, Core, Feature};

pub fn ffi_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("FFI", core);
    feature.define("FEATURE_FFI", "1");
    feature.include("{sources}/ffi/include");

    feature.sources("{sources}/ffi/src/*.c");
    // Single-threaded callout support
    feature.sources("{sources}/ffi/src/sameThread/*.c");
    // Callback support
    feature.sources("{sources}/ffi/src/callbacks/*.c");
    // Required by callbacks
    feature.sources("{sources}/src/semaphores/pharoSemaphore.c");
    feature.sources("{sources}/src/threadSafeQueue/threadSafeQueue.c");

    let ffi_lib = pkg_config::probe_library("libffi").unwrap();
    feature.add_includes(ffi_lib.include_paths);

    feature
}