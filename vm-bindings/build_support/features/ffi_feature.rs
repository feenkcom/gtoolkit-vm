#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Feature, MACOSX_DEPLOYMENT_TARGET};
use clang_sys::support::Clang;
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
    let ffi_binary = match core.target() {
        BuilderTarget::MacOS => ffi_build.join("lib").join("libffi.dylib"),
        BuilderTarget::Linux => ffi_build.join("lib").join("libffi.so"),
        BuilderTarget::Windows => ffi_build.join("bin").join("ffi.dll"),
    };

    if !ffi_build.exists() {
        std::fs::create_dir_all(&ffi_build)?;
    }

    if !ffi_binary.exists() {
        cmake::Config::new(ffi_sources)
            .static_crt(true)
            .out_dir(&ffi_build)
            .define("CMAKE_OSX_DEPLOYMENT_TARGET", MACOSX_DEPLOYMENT_TARGET)
            .build();
    }

    std::fs::copy(
        &ffi_binary,
        core.artefact_directory()
            .join(ffi_binary.file_name().unwrap()),
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
    feature.source("{sources}/ffi/src/sameThread/sameThread.c");
    // Callback support
    feature.source("{sources}/ffi/src/callbacks/callbackPrimitives.c");
    feature.source("{sources}/ffi/src/callbacks/callbacks.c");
    // Required by callbacks
    feature.source("{sources}/src/semaphores/pharoSemaphore.c");
    feature.source("{sources}/src/threadSafeQueue/threadSafeQueue.c");

    if cfg!(target_arch = "x86_64") {
        compile_ffi(core).expect("Failed to compile ffi");
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
    } else if cfg!(target_arch = "aarch64") {
        let clang = Clang::find(None, &[]).unwrap();
        let mut ffi_includes = vec![];
        if let Some(c_search_paths) = clang.c_search_paths {
            for search_path in &c_search_paths {
                if search_path.join("ffi").join("ffi.h").exists() {
                    ffi_includes.push(search_path.clone());
                    ffi_includes.push(search_path.join("ffi"));
                }
            }
        }
        feature.add_includes(ffi_includes);
        feature.dependency(Dependency::Library("ffi".to_string(), vec![]));
    }

    feature
}
