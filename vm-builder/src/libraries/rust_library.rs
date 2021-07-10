use crate::options::FinalOptions;
use crate::{Library, LibraryLocation};
use std::path::PathBuf;
use std::process::Command;
use url::Url;

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

    fn crate_source_directory(&self, options: &FinalOptions) -> PathBuf {
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

    fn force_compile(&self, options: &FinalOptions) {
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

    fn compiled_library(&self, options: &FinalOptions) -> PathBuf {
        #[cfg(target_os = "linux")]
        let binary_name = format!("lib{}.so", &self.name);
        #[cfg(target_os = "macos")]
        let binary_name = format!("lib{}.dylib", &self.name);
        #[cfg(target_os = "windows")]
        let binary_name = format!("{}.dll", &self.name);

        options
            .target_dir()
            .join(options.target().to_string())
            .join(options.profile())
            .join(binary_name)
    }

    fn ensure_requirements(&self) {
        self.requires.iter().for_each(|each| {
            which::which(each).expect(&format!(
                "{} must exist in the system. Make sure it is in the PATH",
                each
            ));
        })
    }
}

impl From<RustLibrary> for Box<dyn Library> {
    fn from(library: RustLibrary) -> Self {
        Box::new(library)
    }
}
