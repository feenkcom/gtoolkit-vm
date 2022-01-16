extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;

use build_support::*;
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

    let mut core = Core::new("PharoVMCore", builder.clone());
    core.add_sources(builder.sources())
        .include("{sources}/extracted/vm/include/common")
        .include("{sources}/extracted/vm/include/osx")
        .include("{sources}/extracted/vm/include/unix")
        .include("{sources}/include")
        .include("{sources}/include/pharovm")
        .add_includes(builder.includes());

    core.define_for_header("dirent.h", "HAVE_DIRENT_H");
    core.define_for_header("features.h", "HAVE_FEATURES_H");
    core.define_for_header("unistd.h", "HAVE_UNISTD_H");
    core.define_for_header("ndir.h", "HAVE_NDIR_H");
    core.define_for_header("sys/ndir.h", "HAVE_SYS_NDIR_H");
    core.define_for_header("sys/dir.h", "HAVE_SYS_DIR_H");
    core.define_for_header("sys/filio.h", "HAVE_SYS_FILIO_H");
    core.define_for_header("sys/time.h", "HAVE_SYS_TIME_H");
    core.define_for_header("execinfo.h", "HAVE_EXECINFO_H");
    core.define_for_header("dlfcn.h", "HAVE_DLFCN_H");

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
    core.add_feature(ffi_feature(&core));
    #[cfg(feature = "threaded_ffi")]
    core.add_feature(threaded_ffi_feature(&core));

    core.compile();

    #[cfg(feature = "b2d_plugin")]
    b2d_plugin(&core).compile();
    #[cfg(feature = "bit_blt_plugin")]
    bit_blt_plugin(&core).compile();
    #[cfg(feature = "dsa_primitives_plugin")]
    dsa_primitives_plugin(&core).compile();
    #[cfg(feature = "file_plugin")]
    file_plugin(&core).compile();
    #[cfg(feature = "file_attributes_plugin")]
    file_attributes_plugin(&core).compile();
    #[cfg(feature = "jpeg_read_writer2_plugin")]
    jpeg_read_writer2_plugin(&core).compile();
    #[cfg(feature = "jpeg_reader_plugin")]
    jpeg_reader_plugin(&core).compile();
    #[cfg(feature = "large_integers_plugin")]
    large_integers_plugin(&core).compile();
    #[cfg(feature = "locale_plugin")]
    locale_plugin(&core).compile();
    #[cfg(feature = "misc_primitive_plugin")]
    misc_primitive_plugin(&core).compile();
    #[cfg(feature = "socket_plugin")]
    socket_plugin(&core).compile();
    #[cfg(feature = "squeak_ssl_plugin")]
    squeak_ssl_plugin(&core).compile();
    #[cfg(feature = "surface_plugin")]
    surface_plugin(&core).compile();
    #[cfg(all(feature = "unix_os_process_plugin", target_family = "unix"))]
    unix_os_process_plugin(&core).compile();
    #[cfg(feature = "uuid_plugin")]
    uuid_plugin(&core).compile();

    builder.link_libraries();
    builder.generate_bindings();
}
