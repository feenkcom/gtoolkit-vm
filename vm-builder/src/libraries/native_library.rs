use crate::options::BundleOptions;
use crate::Library;
use std::error::Error;
use std::path::PathBuf;

pub trait NativeLibrary: Library {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf;
    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf>;
    fn native_library_include_headers(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }
    fn native_library_linker_libraries(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }
    fn pkg_config_directory(&self, _options: &BundleOptions) -> Option<PathBuf> {
        None
    }
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

    pub fn include_headers(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            for each in dependency.native_library_include_headers(options) {
                paths.push(each);
            }
        }
        paths
    }

    pub fn linker_libraries(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            for each in dependency.native_library_linker_libraries(options) {
                paths.push(each);
            }
        }
        paths
    }

    pub fn include_headers_flags(&self, options: &BundleOptions) -> String {
        self.include_headers(options)
            .into_iter()
            .map(|path| format!("-I{}", path.display()))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn linker_libraries_flags(&self, options: &BundleOptions) -> String {
        self.linker_libraries(options)
            .into_iter()
            .map(|path| format!("-L{}", path.display()))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn ensure_sources(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        for dependency in &self.dependencies {
            dependency.ensure_sources(options)?
        }
        Ok(())
    }

    pub fn pkg_config_directories(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            if let Some(ref path) = dependency.pkg_config_directory(options) {
                paths.push(path.clone());
            }
        }
        paths
    }

    pub fn compile(&self, options: &BundleOptions) {
        for dependency in &self.dependencies {
            dependency.compile(options);
        }
    }
}
