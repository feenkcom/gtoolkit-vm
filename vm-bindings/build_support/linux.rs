use crate::build_support::Builder;

use file_matcher::{FileNamed, OneEntry};
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
            .define("FEATURE_LIB_FREETYPE2", "OFF");

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

        vec![
            // core
            FileNamed::exact("libPharoVMCore.so"),
            // plugins
            FileNamed::exact("libB2DPlugin.so"),
            FileNamed::exact("libBitBltPlugin.so"),
            FileNamed::exact("libDSAPrims.so"),
            FileNamed::exact("libFileAttributesPlugin.so"),
            FileNamed::exact("libFilePlugin.so"),
            FileNamed::exact("libJPEGReaderPlugin.so"),
            FileNamed::exact("libJPEGReadWriter2Plugin.so"),
            FileNamed::exact("libLargeIntegers.so"),
            FileNamed::exact("libLocalePlugin.so"),
            FileNamed::exact("libMiscPrimitivePlugin.so"),
            FileNamed::exact("libSocketPlugin.so"),
            FileNamed::exact("libSqueakSSL.so"),
            FileNamed::exact("libSurfacePlugin.so"),
            FileNamed::exact("libUnixOSProcessPlugin.so"),
            FileNamed::exact("libUUIDPlugin.so"),
            // testing
            FileNamed::exact("libTestLibrary.so"),
            // third party
            FileNamed::exact("libffi.so"),
        ]
        .into_iter()
        .map(|each| each.within(self.compiled_libraries_directory()))
        .collect()
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
