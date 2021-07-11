use crate::build_support::builder::Name;
use crate::build_support::Builder;

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::process::Command;

#[derive(Default, Clone)]
pub struct LinuxBuilder {}

impl LinuxBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

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
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF");

        if let Some(vm_maker) = self.vm_maker() {
            config.define("GENERATE_PHARO_VM", vm_maker);
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

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)> {
        assert!(
            self.compiled_libraries_directory().exists(),
            "Must exist: {:?}",
            self.compiled_libraries_directory().display()
        );

        let libs: Vec<(Name, Option<&str>)> = vec![
            // core
            (Name::Exact("libPharoVMCore.so"), None),
            // plugins
            (Name::Exact("libB2DPlugin.so"), None),
            (Name::Exact("libBitBltPlugin.so"), None),
            (Name::Exact("libDSAPrims.so"), None),
            (Name::Exact("libFileAttributesPlugin.so"), None),
            (Name::Exact("libFilePlugin.so"), None),
            (Name::Exact("libJPEGReaderPlugin.so"), None),
            (Name::Exact("libJPEGReadWriter2Plugin.so"), None),
            (Name::Exact("libLargeIntegers.so"), None),
            (Name::Exact("libLocalePlugin.so"), None),
            (Name::Exact("libMiscPrimitivePlugin.so"), None),
            (Name::Exact("libSocketPlugin.so"), None),
            (Name::Exact("libSqueakSSL.so"), None),
            (Name::Exact("libSurfacePlugin.so"), None),
            (Name::Exact("libUnixOSProcessPlugin.so"), None),
            (Name::Exact("libUUIDPlugin.so"), None),
            // testing
            (Name::Exact("libTestLibrary.so"), None),
        ];

        libs.iter()
            .map(|(library, rename)| {
                (
                    library.find_file(&self.compiled_libraries_directory()),
                    rename.map(|name| name.to_string()),
                )
            })
            .filter(|(library, rename)| library.is_some())
            .map(|(library, rename)| (library.unwrap(), rename))
            .collect()
    }
}
