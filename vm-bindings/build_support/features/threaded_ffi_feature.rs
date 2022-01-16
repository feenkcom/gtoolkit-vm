#[cfg(not(feature = "threaded_ffi"))]
compile_error!("threaded_ffi must be enabled for this crate.");

#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{CompilationUnit, Core, Feature};

pub fn threaded_ffi_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("THREADED_FFI", core);
    feature.define("FEATURE_THREADED_FFI", "1");

    feature.sources("{sources}/ffi/src/pThreadedFFI.c");
    feature.sources("{sources}/ffi/src/worker/worker.c");
    feature.sources("{sources}/ffi/src/worker/workerPrimitives.c");
    feature.sources("{sources}/ffi/src/worker/workerTask.c");

    feature
}
