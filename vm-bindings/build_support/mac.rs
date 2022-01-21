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

    fn prepare_environment(&self) {
        if cfg!(target_arch = "x86_64") {
            //config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        } else if cfg!(target_arch = "aarch64") {
            //config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        }
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("osx")
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
