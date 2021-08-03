use crate::options::BundleOptions;
use downloader::{Download, Downloader};
use flate2::read::GzDecoder;
use fs_extra::dir::CopyOptions;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use url::Url;
use user_error::UserFacingError;
use xz2::read::XzDecoder;

pub trait Library: Debug + Send + Sync {
    fn location(&self) -> &LibraryLocation;
    fn name(&self) -> &str;
    fn compiled_library_name(&self) -> &CompiledLibraryName {
        &CompiledLibraryName::Default
    }

    fn source_directory(&self, options: &BundleOptions) -> PathBuf {
        options.third_party_libraries_directory().join(self.name())
    }

    fn ensure_sources(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        let location = self.location();
        location.ensure_sources(&self.source_directory(options), options)
    }

    fn is_compiled(&self, options: &BundleOptions) -> bool {
        self.compiled_library(options).exists()
    }

    fn compile(&self, options: &BundleOptions) {
        self.force_compile(options);
    }

    fn force_compile(&self, options: &BundleOptions);

    fn compiled_library_directories(&self, options: &BundleOptions) -> Vec<PathBuf>;

    fn compiled_library(&self, options: &BundleOptions) -> PathBuf {
        let library_name = self.name();
        let compiled_library_name = self.compiled_library_name();
        for directory in self.compiled_library_directories(options) {
            if let Ok(dir) = directory.read_dir() {
                let libraries = dir
                    .filter(|each| each.is_ok())
                    .map(|each| each.unwrap())
                    .filter(|each| each.path().is_file())
                    .filter(|each| compiled_library_name.matches(library_name, &each.path()))
                    .map(|each| each.path())
                    .collect::<Vec<PathBuf>>();

                if libraries.len() > 0 {
                    return libraries.get(0).unwrap().clone();
                }
            }
        }

        panic!("Could not find a compiled library for {}", self.name())
    }

    fn compiled_library_binary(&self, _options: &BundleOptions) -> Result<PathBuf, Box<dyn Error>> {
        Err(UserFacingError::new("Could not find compiled library").into())
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool;

    fn ensure_requirements(&self, options: &BundleOptions);

    fn clone_library(&self) -> Box<dyn Library>;
}

#[derive(Debug, Clone)]
pub enum LibraryLocation {
    Git(GitLocation),
    Path(PathLocation),
    Tar(TarUrlLocation),
    Multiple(Vec<LibraryLocation>),
}

impl LibraryLocation {
    pub fn ensure_sources(
        &self,
        default_source_directory: &PathBuf,
        options: &BundleOptions,
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
            LibraryLocation::Tar(tar) => tar.ensure_sources(default_source_directory, options),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompiledLibraryName {
    /// same as Library::name
    Default,
    /// find a platform specific library with a name that includes String
    Matching(String),
}

impl CompiledLibraryName {
    fn platform_library_ending(&self) -> String {
        #[cfg(target_os = "linux")]
        let ending = "so";
        #[cfg(target_os = "macos")]
        let ending = "dylib";
        #[cfg(target_os = "windows")]
        let ending = "dll";
        ending.to_string()
    }

    fn platform_library_name(&self, name: &str) -> String {
        #[cfg(target_os = "linux")]
        let binary_name = format!("lib{}.so", name);
        #[cfg(target_os = "macos")]
        let binary_name = format!("lib{}.dylib", name);
        #[cfg(target_os = "windows")]
        let binary_name = format!("{}.dll", name);
        binary_name
    }

    pub fn file_name(&self, library_name: &str) -> String {
        self.platform_library_name(library_name)
    }

    pub fn matches(&self, library_name: &str, path: &PathBuf) -> bool {
        match path.file_name() {
            None => false,
            Some(actual_name) => match actual_name.to_str() {
                None => false,
                Some(actual_name) => match self {
                    CompiledLibraryName::Default => {
                        let expected_name = self.platform_library_name(library_name);
                        actual_name.eq_ignore_ascii_case(&expected_name)
                    }
                    CompiledLibraryName::Matching(substring) => {
                        actual_name.contains(&format!(".{}", self.platform_library_ending()))
                            && actual_name.contains(substring)
                    }
                },
            },
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
        options: &BundleOptions,
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
                .arg("pull")
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
        _options: &BundleOptions,
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

#[derive(Debug, Clone)]
pub struct TarUrlLocation {
    url: String,
    archive: TarArchive,
    sources: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum TarArchive {
    Gz,
    Xz,
}

impl TarUrlLocation {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            archive: TarArchive::Gz,
            sources: None,
        }
    }

    pub fn archive(self, archive: TarArchive) -> Self {
        Self {
            url: self.url,
            archive,
            sources: self.sources,
        }
    }

    pub fn sources(self, folder: impl Into<PathBuf>) -> Self {
        Self {
            url: self.url,
            archive: self.archive,
            sources: Some(folder.into()),
        }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &PathBuf,
        options: &BundleOptions,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = options
            .third_party_libraries_directory()
            .join(default_source_directory);

        if !source_directory.exists() {
            std::fs::create_dir_all(&source_directory)?;

            let mut downloader = Downloader::builder()
                .download_folder(&source_directory)
                .build()?;

            let to_download = Download::new(&self.url);

            let mut result = downloader.download(&[to_download])?;
            let download_result = result.remove(0)?;
            let downloaded_path = download_result.file_name.clone();

            let downloaded_tar = File::open(&downloaded_path)?;

            match self.archive {
                TarArchive::Gz => {
                    let xz = GzDecoder::new(downloaded_tar);
                    let mut archive = Archive::new(xz);
                    archive.unpack(&source_directory)?;
                }
                TarArchive::Xz => {
                    let xz = XzDecoder::new(downloaded_tar);
                    let mut archive = Archive::new(xz);
                    archive.unpack(&source_directory)?;
                }
            }

            std::fs::remove_file(&downloaded_path)?;

            if let Some(ref sources) = self.sources {
                let mut copy_options = CopyOptions::default();
                copy_options.content_only = true;

                fs_extra::dir::copy(
                    source_directory.join(sources),
                    &source_directory,
                    &copy_options,
                )?;

                std::fs::remove_dir_all(source_directory.join(sources))?;
            }
        }
        Ok(())
    }
}
