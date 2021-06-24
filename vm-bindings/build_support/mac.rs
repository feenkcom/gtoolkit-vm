use crate::build_support::Builder;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::process::Command;

#[derive(Default, Clone)]
pub struct MacBuilder {}

impl MacBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

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
        self.output_directory().join("build").join("vm")
    }

    fn generate_sources(&self) {
        Command::new("cmake")
            .arg(self.cmake_build_type())
            .arg("-DCOMPILE_EXECUTABLE=OFF")
            .arg("-S")
            .arg(self.vm_sources_directory())
            .arg("-B")
            .arg(self.output_directory())
            .status()
            .unwrap();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("osx")
    }

    fn generated_config_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("include")
            .join("pharovm")
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

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)> {
        assert!(
            self.compiled_libraries_directory().exists(),
            "Must exist: {:?}",
            self.compiled_libraries_directory().display()
        );

        vec![
            // core
            ("libPharoVMCore.dylib", None),
            // plugins
            ("libB2DPlugin.dylib", None),
            ("libBitBltPlugin.dylib", None),
            ("libDSAPrims.dylib", None),
            ("libFileAttributesPlugin.dylib", None),
            ("libFilePlugin.dylib", None),
            ("libJPEGReaderPlugin.dylib", None),
            ("libJPEGReadWriter2Plugin.dylib", None),
            ("libLargeIntegers.dylib", None),
            ("libLocalePlugin.dylib", None),
            ("libMiscPrimitivePlugin.dylib", None),
            ("libSocketPlugin.dylib", None),
            ("libSqueakSSL.dylib", None),
            ("libSurfacePlugin.dylib", None),
            ("libUnixOSProcessPlugin.dylib", None),
            ("libUUIDPlugin.dylib", None),
            // third party
            ("libcairo.2.dylib", None),
            ("libgit2.1.0.1.dylib", Some("libgit2.dylib")),
            ("libpixman-1.0.dylib", None),
            ("libpng12.0.dylib", None),
            ("libSDL2-2.0.dylib", Some("libSDL2.dylib")),
            // testing
            ("libTestLibrary.dylib", None),
        ]
        .iter()
        .map(|(library, rename)| {
            (
                self.compiled_libraries_directory().join(library),
                rename.map(|name| name.to_string()),
            )
        })
        .collect()
    }
}
