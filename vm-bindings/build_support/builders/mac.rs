use std::path::PathBuf;
use std::rc::Rc;

use platforms::Platform;

use crate::Builder;

#[derive(Clone, Debug)]
pub struct MacBuilder {
    platform: Platform,
}

impl MacBuilder {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

impl Builder for MacBuilder {
    fn platform(&self) -> &Platform {
        &self.platform
    }

    // todo: should it be here?
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
