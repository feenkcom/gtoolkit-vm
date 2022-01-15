extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;

use build_support::*;
use file_matcher::{FileNamed, FilesNamed};
use std::ffi::c_void;
use std::mem;
use std::os::raw::{c_int, c_long, c_longlong};

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn main() {
    for var in std::env::vars() {
        println!("{} = {}", var.0, var.1);
    }

    let builder = match std::env::consts::OS {
        "linux" => LinuxBuilder::default().boxed(),
        "macos" => MacBuilder::default().boxed(),
        "windows" => WindowsBuilder::default().boxed(),
        _ => {
            panic!("The platform you're compiling for is not supported");
        }
    };

    let vmmaker = VMMaker::prepare(builder.clone());
    vmmaker.generate_sources(builder.clone());

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
        .var(
            "BUILT_FROM",
            "v8.6.1-134-g80f73e80e - Commit: 80f73e80e - Date: 2021-08-15 11:16:21 +0200",
        )
        .var("ALWAYS_INTERACTIVE", "OFF");
    config.render();

    let extracted_includes = builder
        .vm_sources_directory()
        .join("extracted")
        .join("vm")
        .join("include");
    let common_include = extracted_includes.join("common");
    let platform_include = extracted_includes.join("osx");
    let include = builder.vm_sources_directory().join("include");
    let pharo_include = include.join("pharovm");

    let mut core = Core::new("PharoVMCore", builder.clone());
    core.files(builder.sources())
        .include(common_include)
        .include(platform_include)
        .include(extracted_includes.join("unix"))
        .include(include)
        .include(pharo_include)
        .includes(builder.includes());

    core.check_include_files("dirent.h", "HAVE_DIRENT_H");
    core.check_include_files("features.h", "HAVE_FEATURES_H");
    core.check_include_files("unistd.h", "HAVE_UNISTD_H");
    core.check_include_files("ndir.h", "HAVE_NDIR_H");
    core.check_include_files("sys/ndir.h", "HAVE_SYS_NDIR_H");
    core.check_include_files("sys/dir.h", "HAVE_SYS_DIR_H");
    core.check_include_files("sys/filio.h", "HAVE_SYS_FILIO_H");
    core.check_include_files("sys/time.h", "HAVE_SYS_TIME_H");
    core.check_include_files("execinfo.h", "HAVE_EXECINFO_H");
    core.check_include_files("dlfcn.h", "HAVE_DLFCN_H");

    core.flag("-Wno-int-conversion");
    core.flag("-Wno-macro-redefined");
    core.flag("-Wno-unused-value");
    core.flag("-Wno-pointer-to-int-cast");
    core.flag("-Wno-non-literal-null-conversion");
    core.flag("-Wno-conditional-type-mismatch");
    core.flag("-Wno-compare-distinct-pointer-types");
    core.flag("-Wno-incompatible-function-pointer-types");
    core.flag("-Wno-pointer-sign");
    core.flag("-Wno-unused-command-line-argument");
    core.flag("-Wno-undef-prefix");

    core.define("IMMUTABILITY", "1");
    core.define("COGMTVM", "0");
    core.define("STACKVM", "0");
    core.define("PharoVM", "1");
    core.define("USE_INLINE_MEMORY_ACCESSORS", "1");
    core.define("ASYNC_FFI_QUEUE", "1");
    core.define("ARCH", "64");
    core.define("VM_LABEL(foo)", "0");
    core.define("SOURCE_PATH_SIZE", "80");
    //core.define("NDEBUG", None);
    //core.define("DEBUGVM", "0");

    // unix
    core.define("LSB_FIRST", "1");
    core.define("OSX", "1");
    core.define("HAVE_TM_GMTOFF", None);

    #[cfg(feature = "ffi")]
    core.define("FEATURE_FFI", "1");
    #[cfg(feature = "threaded_ffi")]
    core.define("FEATURE_THREADED_FFI", "1");

    #[cfg(feature = "ffi")]
    {
        let ffi_lib = pkg_config::probe_library("libffi").unwrap();
        core.includes(ffi_lib.include_paths);
    }

    core.compile();

    file_plugin(core.clone()).compile();
    file_attributes_plugin(core.clone()).compile();
    misc_primitive_plugin(core.clone()).compile();

    builder.link_libraries();
    builder.generate_bindings();
    // builder.export_shared_libraries();
}
