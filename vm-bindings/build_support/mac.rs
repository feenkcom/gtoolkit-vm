use crate::build_support::Builder;

use file_matcher::{FileNamed, OneEntry};
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
            .define("FEATURE_LIB_FREETYPE2", "OFF");

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

        vec![
            // core
            FileNamed::exact("libPharoVMCore.dylib"),
            // plugins
            FileNamed::exact("libB2DPlugin.dylib"),
            FileNamed::exact("libBitBltPlugin.dylib"),
            FileNamed::exact("libDSAPrims.dylib"),
            FileNamed::exact("libFileAttributesPlugin.dylib"),
            FileNamed::exact("libFilePlugin.dylib"),
            FileNamed::exact("libJPEGReaderPlugin.dylib"),
            FileNamed::exact("libJPEGReadWriter2Plugin.dylib"),
            FileNamed::exact("libLargeIntegers.dylib"),
            FileNamed::exact("libLocalePlugin.dylib"),
            FileNamed::exact("libMiscPrimitivePlugin.dylib"),
            FileNamed::exact("libSocketPlugin.dylib"),
            FileNamed::exact("libSqueakSSL.dylib"),
            FileNamed::exact("libSurfacePlugin.dylib"),
            FileNamed::exact("libUnixOSProcessPlugin.dylib"),
            FileNamed::exact("libUUIDPlugin.dylib"),
            FileNamed::exact("libTestLibrary.dylib"),
            // third party
            #[cfg(target_arch = "x86_64")]
            FileNamed::exact("libffi.dylib"),
        ]
        .into_iter()
        .map(|library| library.within(self.compiled_libraries_directory()))
        .collect()
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
