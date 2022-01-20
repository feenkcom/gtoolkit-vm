#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{CompilationUnit, Core, Feature};

pub fn ffi_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("FFI", core);
    feature.define("FEATURE_FFI", "1");
    feature.include("{sources}/ffi/include");

    feature.source("{sources}/ffi/src/functionDefinitionPrimitives.c");
    feature.source("{sources}/ffi/src/primitiveCalls.c");
    feature.source("{sources}/ffi/src/primitiveUtils.c");
    feature.source("{sources}/ffi/src/types.c");
    feature.source("{sources}/ffi/src/typesPrimitives.c");
    feature.source("{sources}/ffi/src/utils.c");
    // Single-threaded callout support
    feature.source("{sources}/ffi/src/sameThread/*.c");
    // Callback support
    feature.source("{sources}/ffi/src/callbacks/*.c");
    // Required by callbacks
    feature.source("{sources}/src/semaphores/pharoSemaphore.c");
    feature.source("{sources}/src/threadSafeQueue/threadSafeQueue.c");

    let ffi_lib = pkg_config::Config::new().statik(true).probe("libffi").unwrap();
    feature.add_includes(ffi_lib.include_paths);

    feature
}
