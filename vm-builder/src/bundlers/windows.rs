use crate::bundlers::Bundler;
use crate::BuildOptions;

pub struct WindowsBundler {}

impl WindowsBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for WindowsBundler {
    fn bundle(&self, configuration: &BuildOptions) {}
}
