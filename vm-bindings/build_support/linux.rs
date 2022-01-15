use crate::build_support::Builder;

use crate::build_support::builder::BuilderTarget;
use file_matcher::OneEntry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct LinuxBuilder;

impl Debug for LinuxBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for LinuxBuilder {
    fn target(&self) -> BuilderTarget {
        BuilderTarget::Linux
    }

    fn platform_extracted_sources(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    fn platform_includes(&self) -> Vec<PathBuf> {
        todo!()
    }

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

    fn compile_sources(&self) {}

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
            self.compiled_libraries_directory(),
        )
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
