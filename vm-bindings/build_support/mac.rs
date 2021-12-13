use crate::build_support::Builder;

use file_matcher::OneEntry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct MacBuilder;

impl Debug for MacBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for MacBuilder {
    fn vm_binary(&self) -> PathBuf {
        self.compiled_libraries_directory()
            .join("libPharoVMCore.dylib")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("vm")
    }

    fn compile_sources(&self) {
        assert!(
            self.vm_sources_directory().exists(),
            "Source directory must exist: {:?}",
            self.vm_sources_directory().display()
        );
        assert!(
            self.output_directory().exists(),
            "Output directory must exist: {:?}",
            self.output_directory().display()
        );

        let mut config = cmake::Config::new(self.vm_sources_directory());
        config
            .no_build_target(true)
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF")
            .define("FEATURE_LIB_CAIRO", "OFF")
            .define("FEATURE_LIB_FREETYPE2", "OFF")
            .define("PHARO_VM_IN_WORKER_THREAD", "OFF");

        config
            .cflag("-Wno-shift-negative-value")
            .cflag("-Wno-int-conversion")
            .cflag("-Wno-unused-function")
            .cflag("-Wno-unused-variable");

        if cfg!(target_arch = "x86_64") {
            config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        } else if cfg!(target_arch = "aarch64") {
            config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        }

        if let Some(vmmaker_vm) = self.vmmaker_vm() {
            config.define("GENERATE_PHARO_VM", vmmaker_vm);
        }
        if let Some(vmmaker_image) = self.vmmaker_image() {
            config.define("GENERATE_PHARO_IMAGE", vmmaker_image);
        }

        config.build();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("osx")
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");

        println!(
            "cargo:rustc-link-search={}",
            self.compiled_libraries_directory().display()
        );
    }

    fn shared_libraries_to_export(&self) -> Vec<OneEntry> {
        assert!(
            self.compiled_libraries_directory().exists(),
            "Must exist: {:?}",
            self.compiled_libraries_directory().display()
        );
        self.filenames_from_libdir(
            vec![
                // core
                "libPharoVMCore.dylib",
                // plugins
                "libB2DPlugin.dylib",
                "libBitBltPlugin.dylib",
                "libDSAPrims.dylib",
                "libFileAttributesPlugin.dylib",
                "libFilePlugin.dylib",
                "libJPEGReaderPlugin.dylib",
                "libJPEGReadWriter2Plugin.dylib",
                "libLargeIntegers.dylib",
                "libLocalePlugin.dylib",
                "libMiscPrimitivePlugin.dylib",
                "libSocketPlugin.dylib",
                "libSqueakSSL.dylib",
                "libSurfacePlugin.dylib",
                "libUnixOSProcessPlugin.dylib",
                "libUUIDPlugin.dylib",
                "libTestLibrary.dylib",
                // third party
                #[cfg(target_arch = "x86_64")]
                "libffi.dylib",
            ], 
            self.compiled_libraries_directory())
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
