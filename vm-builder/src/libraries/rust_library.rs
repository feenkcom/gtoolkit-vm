use crate::options::BundleOptions;
use crate::{Library, LibraryLocation};
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct RustLibrary {
    name: String,
    location: LibraryLocation,
    features: Vec<String>,
    requires: Vec<String>,
}

impl RustLibrary {
    pub fn new(name: &str, location: LibraryLocation) -> Self {
        Self {
            name: name.to_owned(),
            location,
            features: vec![],
            requires: vec![],
        }
    }

    pub fn feature(self, feature: impl Into<String>) -> Self {
        let mut library = self.clone();
        library.features.push(feature.into());
        library
    }

    pub fn requires(self, executable: impl Into<String>) -> Self {
        let mut library = self.clone();
        library.requires.push(executable.into());
        library
    }

    pub fn features(self, features: Vec<&str>) -> Self {
        let mut library = self.clone();
        library.features = features.iter().map(|each| each.to_string()).collect();
        library
    }

    fn crate_source_directory(&self, options: &BundleOptions) -> PathBuf {
        options.third_party_libraries_directory().join(&self.name)
    }
}

impl Library for RustLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn force_compile(&self, options: &BundleOptions) {
        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--target")
            .arg(options.target().to_string())
            .arg("--target-dir")
            .arg(options.target_dir())
            .arg("--manifest-path")
            .arg(self.crate_source_directory(options).join("Cargo.toml"));

        if options.release() {
            command.arg("--release");
        }

        let status = command.status().unwrap();
        if !status.success() {
            panic!("Could not compile {}", self.name);
        }
    }

    fn compiled_library_directories(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let path = options
            .target_dir()
            .join(options.target().to_string())
            .join(options.profile());
        vec![path]
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool {
        false
    }

    fn ensure_requirements(&self, _options: &BundleOptions) {
        self.requires.iter().for_each(|each| {
            which::which(each).expect(&format!(
                "{} must exist in the system. Make sure it is in the PATH",
                each
            ));
        })
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl From<RustLibrary> for Box<dyn Library> {
    fn from(library: RustLibrary) -> Self {
        Box::new(library)
    }
}
