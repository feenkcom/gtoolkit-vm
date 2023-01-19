use crate::Builder;
use platforms::Platform;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct OtherBuilder {
    platform: Platform,
}

impl OtherBuilder {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

impl Builder for OtherBuilder {
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
