use crate::options::BundleOptions;
use crate::Library;
use std::error::Error;
use std::path::PathBuf;

pub trait NativeLibrary: Library {
    /// Return the root build directory of the library.
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf;

    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf>;
    /// Returns a collection of include directories exported by the native library.
    /// Dependent libraries will search headers within these directories
    fn native_library_include_headers(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }
    /// Returns a collection of directories that contain the compiled libraries.
    /// Dependent libraries will search libraries to link within these directories.
    fn native_library_linker_libraries(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }
    /// If a native library creates a pkg-config .pc file, return a directory that contains it
    fn pkg_config_directory(&self, _options: &BundleOptions) -> Option<PathBuf> {
        None
    }

    fn clone_native_library(&self) -> Box<dyn NativeLibrary>;

    fn msvc_include_directories(&self) -> Vec<PathBuf> {
        let msvc = PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30037");
        let sdk = PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0");

        vec![
            msvc.join("include"),
            sdk.join("ucrt"),
            sdk.join("shared"),
            sdk.join("um"),
        ]
    }

    fn msvc_lib_directories(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\um\\x64"),
            PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\ucrt\\x64"),
            PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30037\\lib\\x64")
        ]
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

    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }
}

impl Clone for NativeLibraryDependencies {
    fn clone(&self) -> Self {
        Self {
            dependencies: self
                .dependencies
                .iter()
                .map(|library| library.clone_native_library())
                .collect::<Vec<Box<dyn NativeLibrary>>>(),
        }
    }
}
