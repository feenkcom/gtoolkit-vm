use crate::build_support::Builder;

use file_matcher::{FileNamed, OneFile, OneFileNamed};
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
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF");

        if cfg!(target_arch = "x86_64") {
            config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        } else if cfg!(target_arch = "aarch64") {
            config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        }

        if let Some(vm_maker) = self.vm_maker() {
            config.define("GENERATE_PHARO_VM", vm_maker);
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

    fn shared_libraries_to_export(&self) -> Vec<OneFile> {
        assert!(
            self.compiled_libraries_directory().exists(),
            "Must exist: {:?}",
            self.compiled_libraries_directory().display()
        );

        vec![
            // core
            FileNamed::exact("libPharoVMCore.dylib").boxed(),
            // plugins
            FileNamed::exact("libB2DPlugin.dylib").boxed(),
            FileNamed::exact("libBitBltPlugin.dylib").boxed(),
            FileNamed::exact("libDSAPrims.dylib").boxed(),
            FileNamed::exact("libFileAttributesPlugin.dylib").boxed(),
            FileNamed::exact("libFilePlugin.dylib").boxed(),
            FileNamed::exact("libJPEGReaderPlugin.dylib").boxed(),
            FileNamed::exact("libJPEGReadWriter2Plugin.dylib").boxed(),
            FileNamed::exact("libLargeIntegers.dylib").boxed(),
            FileNamed::exact("libLocalePlugin.dylib").boxed(),
            FileNamed::exact("libMiscPrimitivePlugin.dylib").boxed(),
            FileNamed::exact("libSocketPlugin.dylib").boxed(),
            FileNamed::exact("libSqueakSSL.dylib").boxed(),
            FileNamed::exact("libSurfacePlugin.dylib").boxed(),
            FileNamed::exact("libUnixOSProcessPlugin.dylib").boxed(),
            FileNamed::exact("libUUIDPlugin.dylib").boxed(),
            // third party
            FileNamed::exact("libcairo.2.dylib").boxed(),
            #[cfg(target_arch = "x86_64")]
            FileNamed::wildmatch("libfreetype*.dylib").alias("libfreetype.6.dylib").boxed(),
            FileNamed::exact("libpixman-1.0.dylib").boxed(),
            FileNamed::any(vec!["libpng12.0.dylib", "libpng16.0.dylib"]).boxed(),
            // testing
            FileNamed::exact("libTestLibrary.dylib").boxed(),
        ]
        .into_iter()
        .map(|library| library.within_path_buf(self.compiled_libraries_directory()))
        .collect()
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
