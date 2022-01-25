#[cfg(not(feature = "threaded_ffi"))]
compile_error!("threaded_ffi must be enabled for this crate.");

#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{ffi_feature, CompilationUnit, Core, Dependency, Feature};

pub fn threaded_ffi_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("THREADED_FFI", core);
    feature.dependency(Dependency::Feature(ffi_feature(core)));
    feature.define("FEATURE_THREADED_FFI", "1");

    feature.source("{sources}/ffi/src/pThreadedFFI.c");
    feature.source("{crate}/patched/worker.c");
    feature.source("{sources}/ffi/src/worker/workerPrimitives.c");
    feature.source("{sources}/ffi/src/worker/workerTask.c");

    feature
}
