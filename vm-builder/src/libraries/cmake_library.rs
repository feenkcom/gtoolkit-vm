use crate::options::{BundleOptions, Target};
use crate::{
    CompiledLibraryName, Library, LibraryLocation, NativeLibrary, NativeLibraryDependencies,
};
use file_matcher::FileNamed;
use rustc_version::version_meta;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CMakeLibrary {
    name: String,
    compiled_name: CompiledLibraryName,
    location: LibraryLocation,
    defines: Vec<(String, String)>,
    dependencies: NativeLibraryDependencies,
    files_to_delete: Vec<FileNamed>,
}

impl CMakeLibrary {
    pub fn new(name: &str, location: LibraryLocation) -> Self {
        Self {
            name: name.to_owned(),
            compiled_name: CompiledLibraryName::Default,
            location,
            defines: vec![],
            dependencies: NativeLibraryDependencies::new(),
            files_to_delete: vec![],
        }
    }

    pub fn define(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let mut defines = self.defines;
        defines.push((define.into(), value.into()));
        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines,
            dependencies: self.dependencies,
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn depends(self, library: Box<dyn NativeLibrary>) -> Self {
        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies.add(library),
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn compiled_name(self, compiled_name: CompiledLibraryName) -> Self {
        Self {
            name: self.name,
            compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies,
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn delete(self, entry_to_delete: impl Into<FileNamed>) -> Self {
        let mut entries = self.files_to_delete;
        entries.push(entry_to_delete.into());

        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies,
            files_to_delete: entries,
        }
    }
}

impl NativeLibrary for CMakeLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        options.target_dir().join(self.name())
    }

    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf> {
        self.dependencies.dependency_prefixes(options)
    }

    fn pkg_config_directory(&self, options: &BundleOptions) -> Option<PathBuf> {
        let directory = self
            .native_library_prefix(options)
            .join("lib")
            .join("pkgconfig");

        if directory.exists() {
            return Some(directory);
        }

        let directory = self
            .native_library_prefix(options)
            .join("share")
            .join("pkgconfig");

        if directory.exists() {
            return Some(directory);
        }

        None
    }

    fn clone_native_library(&self) -> Box<dyn NativeLibrary> {
        Box::new(Clone::clone(self))
    }
}

impl Library for CMakeLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn compiled_library_name(&self) -> &CompiledLibraryName {
        &self.compiled_name
    }

    fn ensure_sources(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        self.location()
            .ensure_sources(&self.source_directory(options), options)?;
        self.dependencies.ensure_sources(options)?;
        Ok(())
    }

    fn force_compile(&self, options: &BundleOptions) {
        self.dependencies.compile(options);

        let mut config = cmake::Config::new(self.source_directory(options));

        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir).expect(&format!("Could not create {:?}", &out_dir));
        }

        config
            .static_crt(true)
            .target(&options.target().to_string())
            .host(&version_meta().unwrap().host)
            .out_dir(&out_dir)
            .profile(&options.profile());

        println!(
            "Building CMake library for target = {:?} and host = {:?}",
            &options.target().to_string(),
            &version_meta().unwrap().host
        );

        let mut cmake_prefix_paths = self.native_library_dependency_prefixes(options);
        if let Ok(ref path) = std::env::var("CMAKE_PREFIX_PATH") {
            cmake_prefix_paths.push(Path::new(path).to_path_buf());
        }

        let cmake_prefix_path = cmake_prefix_paths
            .into_iter()
            .map(|each| each.into_os_string().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(";");

        config.define("CMAKE_PREFIX_PATH", &cmake_prefix_path);

        match options.target() {
            Target::X8664appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
            }
            Target::AArch64appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
            }
            Target::X8664pcWindowsMsvc => {}
            Target::X8664UnknownlinuxGNU => {}
        }

        let ld_library_paths = self
            .native_library_dependency_prefixes(options)
            .into_iter()
            .map(|each| each.join("lib"))
            .collect::<Vec<PathBuf>>();

        for library_path in &ld_library_paths {
            config.cflag(format!("-L{}", library_path.display()));
        }

        for define in &self.defines {
            config.define(&define.0, &define.1);
        }

        config.build();

        for entry_to_delete in &self.files_to_delete {
            let lib = entry_to_delete.within(out_dir.join("lib"));
            std::fs::remove_file(lib.as_path_buf().unwrap()).unwrap();
        }
    }

    fn compiled_library_directories(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let lib_dir = self.native_library_prefix(options).join("lib");
        let bin_dir = self.native_library_prefix(options).join("bin");
        vec![lib_dir, bin_dir]
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool {
        !self.dependencies.is_empty()
    }

    fn ensure_requirements(&self, _options: &BundleOptions) {
        which::which("pkg-config")
            .expect("CMake projects require pkg-config, make sure it is installed");
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl From<CMakeLibrary> for Box<dyn Library> {
    fn from(library: CMakeLibrary) -> Self {
        Box::new(library)
    }
}

impl From<CMakeLibrary> for Box<dyn NativeLibrary> {
    fn from(library: CMakeLibrary) -> Self {
        Box::new(library)
    }
}
