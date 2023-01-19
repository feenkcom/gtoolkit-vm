use std::path::PathBuf;
use std::rc::Rc;

use platforms::Platform;

use crate::Builder;

#[derive(Clone, Debug)]
pub struct LinuxBuilder {
    platform: Platform,
}

impl LinuxBuilder {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

impl Builder for LinuxBuilder {
    fn platform(&self) -> &Platform {
        &self.platform
    }

    fn prepare_environment(&self) {}

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("unix")
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
