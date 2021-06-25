use crate::bundlers::Bundler;
use crate::BuildOptions;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct LinuxBundler {}

impl LinuxBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for LinuxBundler {
    fn bundle(&self, _configuration: &BuildOptions) {}
}
