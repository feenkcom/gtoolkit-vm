use crate::build_support::Builder;

use file_matcher::OneEntry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct LinuxBuilder;

impl Debug for LinuxBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for LinuxBuilder {
    fn vm_binary(&self) -> PathBuf {
        self.compiled_libraries_directory()
            .join("libPharoVMCore.so")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("vm")
    }

    fn compile_sources(&self) {
        let mut config = cmake::Config::new(self.vm_sources_directory());
        config
            .no_build_target(true)
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF")
            .define("FEATURE_LIB_CAIRO", "OFF")
            .define("FEATURE_LIB_FREETYPE2", "OFF")
            .define("PHARO_VM_IN_WORKER_THREAD", "OFF");

        if let Some(vmmaker_vm) = self.vmmaker_vm() {
            config.define("GENERATE_PHARO_VM", vmmaker_vm);
        }
        if let Some(vmmaker_image) = self.vmmaker_image() {
            config.define("GENERATE_PHARO_IMAGE", vmmaker_image);
        }
        config.build();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("unix")
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");
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
                "libPharoVMCore.so",
                // plugins
                "libB2DPlugin.so",
                "libBitBltPlugin.so",
                "libDSAPrims.so",
                "libFileAttributesPlugin.so",
                "libFilePlugin.so",
                "libJPEGReaderPlugin.so",
                "libJPEGReadWriter2Plugin.so",
                "libLargeIntegers.so",
                "libLocalePlugin.so",
                "libMiscPrimitivePlugin.so",
                "libSocketPlugin.so",
                "libSqueakSSL.so",
                "libSurfacePlugin.so",
                "libUnixOSProcessPlugin.so",
                "libUUIDPlugin.so",
                // testing
                "libTestLibrary.so",
                // third party
                "libffi.so",
            ],
            self.compiled_libraries_directory())
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
