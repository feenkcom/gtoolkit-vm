use crate::build_support::Builder;

use crate::BuilderTarget;
use file_matcher::OneEntry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct MacBuilder;

impl Debug for MacBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for MacBuilder {
    fn target(&self) -> BuilderTarget {
        BuilderTarget::MacOS
    }

    fn platform_extracted_sources(&self) -> Vec<PathBuf> {
        let root = self.vm_sources_directory();

        [
            // Common sources
            root.join("extracted/vm/src/common/sqHeapMap.c"),
            root.join("extracted/vm/src/common/sqVirtualMachine.c"),
            root.join("extracted/vm/src/common/sqNamedPrims.c"),
            root.join("extracted/vm/src/common/sqExternalSemaphores.c"),
            root.join("extracted/vm/src/common/sqTicker.c"),
            // Platform sources
            root.join("extracted/vm/src/osx/aioOSX.c"),
            root.join("src/debugUnix.c"),
            root.join("src/utilsMac.mm"),
            // Support sources
            root.join("src/fileDialogMac.m"),
            // Virtual Memory functions
            root.join("src/memoryUnix.c"),
        ]
        .to_vec()
    }

    fn platform_includes(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn vm_binary(&self) -> PathBuf {
        self.compiled_libraries_directory()
            .join("libPharoVMCore.dylib")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
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

        config.build();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("osx")
    }

    fn link_libraries(&self) {
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
            self.compiled_libraries_directory(),
        )
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
