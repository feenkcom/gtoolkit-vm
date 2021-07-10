use crate::options::FinalOptions;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use url::Url;
use user_error::{UserFacingError, UFE};

pub trait Library {
    fn location(&self) -> &LibraryLocation;
    fn name(&self) -> &str;

    fn source_directory(&self, options: &FinalOptions) -> PathBuf {
        options.third_party_libraries_directory().join(self.name())
    }

    fn ensure_sources(&self, options: &FinalOptions) -> Result<(), Box<dyn Error>> {
        let location = self.location();
        location.ensure_sources(&self.source_directory(options), options)
    }

    fn is_compiled(&self, options: &FinalOptions) -> bool {
        self.compiled_library(options).exists()
    }

    fn compile(&self, options: &FinalOptions) {
        self.force_compile(options);
    }

    fn force_compile(&self, options: &FinalOptions);

    fn compiled_library(&self, options: &FinalOptions) -> PathBuf;

    fn ensure_requirements(&self);
}

#[derive(Debug, Clone)]
pub enum LibraryLocation {
    Git(GitLocation),
    Path(PathLocation),
    Multiple(Vec<LibraryLocation>),
}

impl LibraryLocation {
    pub fn ensure_sources(
        &self,
        default_source_directory: &PathBuf,
        options: &FinalOptions,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            LibraryLocation::Git(git_location) => {
                git_location.ensure_sources(default_source_directory, options)
            }
            LibraryLocation::Path(path_location) => {
                path_location.ensure_sources(default_source_directory, options)
            }
            LibraryLocation::Multiple(locations) => {
                let mut iterator = locations.iter();
                while let Some(location) = iterator.next() {
                    location.ensure_sources(default_source_directory, options)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitLocation {
    repository: Url,
    version: GitVersion,
    directory: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum GitVersion {
    Tag(String),
    Commit(String),
    Branch(String),
    Latest,
}

impl GitLocation {
    pub fn new(repository: &str) -> Self {
        Self {
            repository: Url::parse(repository).unwrap(),
            version: GitVersion::Latest,
            directory: None,
        }
    }

    pub fn commit(self, commit: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Commit(commit.into()),
            directory: self.directory,
        }
    }

    pub fn branch(self, branch: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Branch(branch.into()),
            directory: self.directory,
        }
    }

    pub fn tag(self, tag: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Tag(tag.into()),
            directory: self.directory,
        }
    }

    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            repository: self.repository,
            version: self.version,
            directory: Some(directory.into()),
        }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &PathBuf,
        options: &FinalOptions,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = match self.directory {
            None => options
                .third_party_libraries_directory()
                .join(default_source_directory),
            Some(ref custom_directory) => options
                .third_party_libraries_directory()
                .join(custom_directory),
        };

        if !source_directory.exists() {
            let result = Command::new("git")
                .arg("clone")
                .arg(self.repository.to_string())
                .arg(&source_directory)
                .status()
                .unwrap();

            if !result.success() {
                return Err(Box::new(
                    UserFacingError::new("Failed to build project")
                        .reason(format!("Could not clone {}", &self.repository))
                        .help(
                            "Make sure the configuration is correct and the git repository exists",
                        ),
                ));
            }
        }

        Command::new("git")
            .current_dir(&source_directory)
            .arg("clean")
            .arg("-fdx")
            .status()
            .unwrap();

        Command::new("git")
            .current_dir(&source_directory)
            .arg("fetch")
            .arg("--all")
            .arg("--tags")
            .status()
            .unwrap();

        let status = match &self.version {
            GitVersion::Tag(tag) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(format!("tags/{}", tag))
                .status()
                .unwrap(),
            GitVersion::Commit(commit) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(commit)
                .status()
                .unwrap(),
            GitVersion::Branch(branch) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(branch)
                .status()
                .unwrap(),
            GitVersion::Latest => Command::new("git")
                .current_dir(&source_directory)
                .arg("pull.ff")
                .status()
                .unwrap(),
        };

        if !status.success() {
            return Err(Box::new(
                UserFacingError::new("Failed to build project")
                    .reason(format!(
                        "Could not checkout {:?} of {:?}",
                        &self.version, &self.repository
                    ))
                    .help("Make sure the configuration is correct and the git repository exists"),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PathLocation {
    path: PathBuf,
}

impl PathLocation {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &PathBuf,
        options: &FinalOptions,
    ) -> Result<(), Box<dyn Error>> {
        if !default_source_directory.exists() {
            return Err(Box::new(
                UserFacingError::new("Failed to build project")
                    .reason(format!(
                        "{} sources directory does not exist",
                        self.path.display()
                    ))
                    .help("Make sure the configuration is correct and the sources exist"),
            ));
        }
        Ok(())
    }
}
