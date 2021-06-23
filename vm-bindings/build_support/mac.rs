use crate::build_support::Builder;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct MacBuilder {}

impl Debug for MacBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for MacBuilder {
    fn vm_binary(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("dist")
            .join("Pharo.app")
            .join("Contents")
            .join("MacOS")
            .join("Pharo")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("dist")
            .join("Pharo.app")
            .join("Contents")
            .join("MacOS")
            .join("Plugins")
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("osx")
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");

        println!(
            "cargo:rustc-link-search={}/build/dist/Pharo.app/Contents/MacOS/Plugins",
            self.output_directory().display()
        );
    }

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)> {
        let mut libraries: Vec<(PathBuf, Option<String>)> = self
            .compiled_libraries_directory()
            .read_dir()
            .unwrap()
            .map(|each| (each.unwrap().path(), None))
            .collect();

        libraries.push((
            self.output_directory()
                .join("build")
                .join("dist")
                .join("libSDL2-2.0d.dylib"),
            Some("libSDL2-2.0.0.dylib".to_string()),
        ));
        libraries
    }
}
