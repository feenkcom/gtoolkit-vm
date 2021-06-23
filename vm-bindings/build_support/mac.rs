use crate::build_support::Builder;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct MacBuilder {}

impl MacBuilder {
    fn compiled_pharo_app(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("vm")
            .join("Debug")// <=== it is hardcoded in CMakeLists.txt on the pharo side, it does not mean that it is compiled in debug mode
            .join("Pharo.app")
    }
}

impl Debug for MacBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl Builder for MacBuilder {
    fn vm_binary(&self) -> PathBuf {
        self.compiled_pharo_app()
            .join("Contents")
            .join("MacOS")
            .join("Pharo")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.compiled_pharo_app()
            .join("Contents")
            .join("MacOS")
            .join("Plugins")
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
            "cargo:rustc-link-search={}/Contents/MacOS/Plugins",
            self.compiled_pharo_app().display()
        );
    }

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)> {
        assert!(self.compiled_pharo_app().exists(), "Must exist: {:?}", self.compiled_pharo_app().display());
        assert!(self.compiled_libraries_directory().exists(), "Must exist: {:?}", self.compiled_libraries_directory().display());

        let mut libraries: Vec<(PathBuf, Option<String>)> = self
            .compiled_libraries_directory()
            .read_dir()
            .unwrap()
            .map(|each| (each.unwrap().path(), None))
            .collect();

        libraries.push((
            self.output_directory()
                .join("build")
                .join("vm")
                .join(self.profile())
                .join("libSDL2-2.0d.dylib"),
            Some("libSDL2-2.0.0.dylib".to_string()),
        ));
        libraries
    }
}
