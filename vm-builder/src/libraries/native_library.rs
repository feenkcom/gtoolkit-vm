use crate::options::BundleOptions;
use crate::Library;
use std::error::Error;
use std::path::PathBuf;

pub trait NativeLibrary: Library {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf;
    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf>;
}

#[derive(Debug)]
pub struct NativeLibraryDependencies {
    dependencies: Vec<Box<dyn NativeLibrary>>,
}

impl NativeLibraryDependencies {
    pub fn new() -> Self {
        Self {
            dependencies: vec![],
        }
    }

    pub fn add(self, dependency: Box<dyn NativeLibrary>) -> Self {
        let mut dependencies = self.dependencies;
        dependencies.push(dependency);
        Self { dependencies }
    }

    pub fn dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            for each in dependency.native_library_dependency_prefixes(options) {
                paths.push(each);
            }
            paths.push(dependency.native_library_prefix(options));
        }
        paths
    }

    pub fn ensure_sources(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        for dependency in &self.dependencies {
            dependency.ensure_sources(options)?
        }
        Ok(())
    }

    pub fn compile(&self, options: &BundleOptions) {
        for dependency in &self.dependencies {
            dependency.compile(options);
        }
    }
}
