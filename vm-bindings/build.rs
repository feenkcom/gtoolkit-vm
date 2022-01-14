extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;

use build_support::*;
use file_matcher::FilesNamed;
use std::env::join_paths;
use std::ffi::c_void;
use std::mem;
use std::os::raw::{c_int, c_long, c_longlong};

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn main() {
    let builder = match std::env::consts::OS {
        "linux" => LinuxBuilder::default().boxed(),
        "macos" => MacBuilder::default().boxed(),
        "windows" => WindowsBuilder::default().boxed(),
        _ => {
            panic!("The platform you're compiling for is not supported");
        }
    };

    let vmmaker = VMMaker::prepare(&builder);
    vmmaker.generate_sources(&builder);

    let mut config = ConfigTemplate::new(
        builder
            .vm_sources_directory()
            .join("include")
            .join("pharovm")
            .join("config.h.in"),
        builder
            .output_directory()
            .join("generated")
            .join("64")
            .join("vm")
            .join("include")
            .join("config.h"),
    );
    config
        .var("VM_NAME", "Pharo")
        .var("DEFAULT_IMAGE_NAME", "Pharo.image")
        .var("OS_TYPE", "Mac OS")
        .var("VM_TARGET", "Darwin-19.4.0")
        .var("VM_TARGET_OS", "1000")
        .var("VM_TARGET_CPU", "x86_64")
        .var("SIZEOF_INT", mem::size_of::<c_int>().to_string())
        .var("SIZEOF_LONG", mem::size_of::<c_long>().to_string())
        .var("SIZEOF_LONG_LONG", mem::size_of::<c_longlong>().to_string())
        .var("SIZEOF_VOID_P", mem::size_of::<*const c_void>().to_string())
        .var("SQUEAK_INT64_TYPEDEF", "long")
        .var("VERSION_MAJOR", "0")
        .var("VERSION_MINOR", "0")
        .var("VERSION_PATCH", "1")
        .var("BUILT_FROM", "v8.6.1-134-g80f73e80e - Commit: 80f73e80e - Date: 2021-08-15 11:16:21 +0200")
        .var("ALWAYS_INTERACTIVE", "OFF");
    config.render();

    let generated = builder
        .output_directory()
        .join("generated")
        .join("64")
        .join("vm");
    let generated_include = generated.join("include");

    let extracted_includes = builder
        .vm_sources_directory()
        .join("extracted")
        .join("vm")
        .join("include");
    let common_include = extracted_includes.join("common");
    let platform_include = extracted_includes.join("osx");
    let include = builder.vm_sources_directory().join("include");
    let pharo_include = include.join("pharovm");

    let original_sources = builder.sources();
    let mut sources = Vec::new();
    let dst = builder.output_directory();
    for file in original_sources.iter() {
        let obj = dst.join(file);
        let obj = if !obj.starts_with(&dst) {
            let source = obj.strip_prefix(builder.vm_sources_directory()).unwrap();
            let dst_source = dst.join(source);
            std::fs::create_dir_all(dst_source.parent().unwrap()).unwrap();
            std::fs::copy(file, &dst_source).unwrap();
            dst_source
        } else {
            obj
        };
        sources.push(obj);
    }

    let mut build = cc::Build::new();
    build
        .static_crt(true)
        .static_flag(true)
        .shared_flag(false)
        .files(sources)
        .include(generated_include)
        .include(common_include)
        .include(platform_include)
        .include(include)
        .include(pharo_include)
        .includes(builder.includes())
        .warnings(false)
        .extra_warnings(false);

    build.flag("-Wno-int-conversion");
    build.flag("-Wno-macro-redefined");
    build.flag("-Wno-unused-value");
    build.flag("-Wno-pointer-to-int-cast");
    build.flag("-Wno-non-literal-null-conversion");
    build.flag("-Wno-conditional-type-mismatch");
    build.flag("-Wno-compare-distinct-pointer-types");
    build.flag("-Wno-incompatible-function-pointer-types");
    build.flag("-Wno-pointer-sign");

    build.define("IMMUTABILITY", "1");
    build.define("COGMTVM", "0");
    build.define("STACKVM", "0");
    build.define("PharoVM", "1");
    build.define("USE_INLINE_MEMORY_ACCESSORS", "1");
    //build.define("ASYNC_FFI_QUEUE", "1");
    build.define("ARCH", "64");
    build.define("VM_LABEL(foo)", "0");
    build.define("SOURCE_PATH_SIZE", "80");
    //build.define("NDEBUG", None);
    //build.define("DEBUGVM", "0");

    // unix
    build.define("LSB_FIRST", "1");
    build.define("OSX", "1");

    #[cfg(feature = "ffi")]
    build.define("FEATURE_FFI", "1");
    #[cfg(feature = "threaded_ffi")]
    build.define("FEATURE_THREADED_FFI", "1");

    #[cfg(feature = "ffi")]
    {
        let ffi_lib = pkg_config::probe_library("libffi").unwrap();
        build.includes(ffi_lib.include_paths);
    }

    build.compile("PharoVMCore");

    // println!("About to build a vm using {:?}", &builder);
    // builder.ensure_build_tools();
    //
    // builder.compile_sources();
    //
    // if !builder.is_compiled() {
    //     panic!("Failed to compile {:?}", builder.vm_binary().display())
    // }
    //
    builder.link_libraries();
    builder.generate_bindings();
    // builder.export_shared_libraries();
}
