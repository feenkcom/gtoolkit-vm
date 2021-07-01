use crate::options::FinalOptions;
use crate::Library;
use std::path::PathBuf;
use std::process::Command;
use url::Url;

#[derive(Clone, Debug)]
pub struct RustLibrary {
    name: String,
    repository: Url,
    commit: Option<String>,
    features: Vec<String>,
    requires: Vec<String>,
}

impl RustLibrary {
    pub fn new(name: &str, repository: &str) -> Self {
        Self {
            name: name.to_owned(),
            repository: Url::parse(repository).unwrap(),
            commit: None,
            features: vec![],
            requires: vec![],
        }
    }

    pub fn commit(self, commit: impl Into<String>) -> Self {
        let mut library = self.clone();
        library.commit = Some(commit.into());
        library
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
    fn is_downloaded(&self, options: &FinalOptions) -> bool {
        self.crate_source_directory(options).exists()
    }

    fn force_download(&self, options: &FinalOptions) {
        let result = Command::new("git")
            .arg("clone")
            .arg(self.repository.to_string())
            .arg(self.crate_source_directory(options))
            .status()
            .unwrap();

        if !result.success() {
            panic!("Could not clone {:?}", self.repository.to_string())
        }
    }

    fn checkout(&self, options: &FinalOptions) {
        Command::new("git")
            .current_dir(self.crate_source_directory(options))
            .arg("clean")
            .arg("-fdx")
            .status()
            .unwrap();

        if let Some(ref commit) = self.commit {
            Command::new("git")
                .current_dir(self.crate_source_directory(options))
                .arg("checkout")
                .arg(commit)
                .status()
                .unwrap();
        } else {
            Command::new("git")
                .current_dir(self.crate_source_directory(options))
                .arg("pull")
                .status()
                .unwrap();
        }
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
