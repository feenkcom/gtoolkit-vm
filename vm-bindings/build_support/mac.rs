use crate::{Builder, BuilderTarget};
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
        println!("cargo:rustc-link-lib=PharoVMCore");
        println!(
            "cargo:rustc-link-search={}",
            self.artefact_directory().display()
        );
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
