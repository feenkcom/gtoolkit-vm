#[cfg(not(feature = "vm_in_worker_thread"))]
compile_error!("vm_in_worker_thread must be enabled for this crate.");

#[cfg(not(feature = "threaded_ffi"))]
compile_error!("threaded_ffi must be enabled for this crate.");

use crate::{threaded_ffi_feature, CompilationUnit, Core, Dependency, Feature};

pub fn vm_in_worker_thread_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("VM_IN_WORKER_THREAD", core);
    feature.dependency(Dependency::Feature(threaded_ffi_feature(core)));
    feature.define("PHARO_VM_IN_WORKER_THREAD", "1");
    feature
}
