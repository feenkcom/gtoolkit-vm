#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Feature};
use std::process::Command;

fn compile_ffi(core: &Core) -> anyhow::Result<()> {
    let ffi_sources = core.output_directory().join("libffi");

    if !ffi_sources.exists() {
        Command::new("git")
            .current_dir(core.output_directory())
            .arg("clone")
            .arg("https://github.com/syrel/libffi.git")
            .status()?;

        // checkout the version of libffi that works
        Command::new("git")
            .current_dir(&ffi_sources)
            .arg("checkout")
            .arg("af975d04e64dfc2116078160b3b75524eb6bf241")
            .status()?;
    }

    let ffi_build = core.output_directory().join("libffi-build");
    if !ffi_build.exists() {
        std::fs::create_dir_all(&ffi_build)?;
    }
    cmake::Config::new(ffi_sources)
        .static_crt(true)
        .out_dir(&ffi_build)
        .build();

    let ffi_binary_name = match core.target() {
        BuilderTarget::MacOS => "libffi.dylib",
        BuilderTarget::Linux => "libffi.so",
        BuilderTarget::Windows => "ffi.dll",
    };

    std::fs::copy(
        ffi_build.join("bin").join(ffi_binary_name),
        core.artefact_directory().join(ffi_binary_name),
    )?;

    Ok(())
}

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

    let lib_ffi_include = feature
        .builder()
        .output_directory()
        .join("libffi-build")
        .join("include");
    let lib_ffi = feature
        .builder()
        .output_directory()
        .join("libffi-build")
        .join("lib");
    feature.add_include(&lib_ffi_include);
    feature.dependency(Dependency::Library("ffi".to_string(), vec![lib_ffi]));

    feature
}
