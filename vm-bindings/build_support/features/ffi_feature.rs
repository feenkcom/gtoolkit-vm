#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{CompilationUnit, Core, Feature, Dependency};

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

    let lib_ffi_include = feature.builder().output_directory().join("libffi-build").join("include");
    let lib_ffi = feature.builder().output_directory().join("libffi-build").join("lib");
    feature.add_include(&lib_ffi_include);
    feature.dependency(Dependency::Library("ffi".to_string(), vec![lib_ffi]));

    feature
}
