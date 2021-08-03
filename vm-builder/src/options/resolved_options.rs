use crate::{BuildOptions, Library, Target};
use feenk_releaser::{Version, VersionBump};
use std::path::{Path, PathBuf};

const DEFAULT_BUILD_DIR: &str = "target";

#[derive(Debug)]
pub struct ResolvedOptions {
    options: BuildOptions,
    target_dir: PathBuf,
    target: Target,
    identifier: String,
    app_name: String,
    executable_name: String,
    version: Version,
    icons: Vec<PathBuf>,
    libraries: Vec<Box<dyn Library>>,
}

impl ResolvedOptions {
    pub fn new(options: BuildOptions) -> Self {
        let target_dir: PathBuf = options.target_dir().map_or_else(
            || {
                options
                    .workspace_directory()
                    .map_or(PathBuf::from(DEFAULT_BUILD_DIR), |workspace| {
                        workspace.join(DEFAULT_BUILD_DIR)
                    })
            },
            |target_dir| target_dir.to_path_buf(),
        );

        let target = options.target();

        let app_name = options
            .app_name()
            .map_or("VM".to_owned(), |name| name.to_owned());

        let identifier = options
            .identifier()
            .map_or_else(|| app_name.clone(), |identifier| identifier.to_owned());

        let executable_name = options
            .executable_name()
            .map_or_else(|| app_name.clone(), |name| name.to_owned());

        let version = options.version().map_or_else(
            || Version::new(VersionBump::Patch),
            |version| {
                Version::parse(version).expect(&format!("Could not parse version {}", version))
            },
        );

        let icons = options.icons().map_or(vec![], |icons| {
            icons
                .iter()
                .map(|icon| PathBuf::from(icon))
                .collect::<Vec<PathBuf>>()
        });

        let libraries = options.libraries().map_or(vec![], |libraries| {
            libraries
                .iter()
                .map(|each| each.as_library())
                .collect::<Vec<Box<dyn Library>>>()
        });

        Self {
            options,
            target_dir,
            target,
            app_name,
            identifier,
            executable_name,
            version,
            icons,
            libraries,
        }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn target_dir(&self) -> &PathBuf {
        &self.target_dir
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn app_name(&self) -> &str {
        self.app_name.as_str()
    }

    pub fn executable_name(&self) -> &str {
        self.executable_name.as_str()
    }

    pub fn executable_extension(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        return None;
        #[cfg(target_os = "macos")]
        return None;
        #[cfg(target_os = "windows")]
        return Some("exe".to_string());
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn verbose(&self) -> i32 {
        self.options.verbose()
    }

    pub fn release(&self) -> bool {
        self.options.release()
    }

    pub fn icons(&self) -> &Vec<PathBuf> {
        &self.icons
    }

    pub fn bundle_dir(&self) -> Option<&Path> {
        self.options.bundle_dir()
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.options.vmmaker_vm()
    }

    pub fn libraries(&self) -> &Vec<Box<dyn Library>> {
        &self.libraries
    }

    pub fn workspace_directory(&self) -> Option<PathBuf> {
        self.options.workspace_directory()
    }
}

impl Clone for ResolvedOptions {
    fn clone(&self) -> Self {
        Self {
            options: self.options.clone(),
            target_dir: self.target_dir.clone(),
            target: self.target.clone(),
            identifier: self.identifier.clone(),
            app_name: self.app_name.clone(),
            executable_name: self.executable_name.clone(),
            version: self.version.clone(),
            icons: self.icons.clone(),
            libraries: self
                .libraries
                .iter()
                .map(|library| library.clone_library())
                .collect(),
        }
    }
}
