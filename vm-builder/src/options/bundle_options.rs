use crate::{Library, ResolvedOptions, Target};
use feenk_releaser::Version;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum Executable {
    App,
    Cli,
}

impl Executable {
    pub fn cargo_bin_name(&self) -> &str {
        match self {
            Executable::App => "vm_client",
            Executable::Cli => "vm_client-cli",
        }
    }

    pub fn compiled_name(&self, options: &ResolvedOptions) -> String {
        let mut executable_name = self.cargo_bin_name().to_owned();

        if let Some(extension) = options.executable_extension() {
            executable_name = format!("{}.{}", &executable_name, &extension);
        };
        executable_name
    }

    pub fn bundled_name(&self, options: &ResolvedOptions) -> String {
        let mut executable_name = match self {
            Executable::App => options.executable_name().to_owned(),
            Executable::Cli => {
                format!("{}-cli", options.executable_name())
            }
        };

        if let Some(extension) = options.executable_extension() {
            executable_name = format!("{}.{}", &executable_name, &extension);
        };
        executable_name
    }
}

#[derive(Debug, Clone)]
pub struct BundleOptions {
    options: ResolvedOptions,
    executables: Vec<Executable>,
}

impl BundleOptions {
    pub fn new(options: ResolvedOptions, executables: Vec<Executable>) -> Self {
        Self {
            options,
            executables,
        }
    }

    pub fn executables(&self) -> &Vec<Executable> {
        &self.executables
    }

    pub fn target(&self) -> &Target {
        self.options.target()
    }

    pub fn target_dir(&self) -> &PathBuf {
        self.options.target_dir()
    }

    pub fn verbose(&self) -> i32 {
        self.options.verbose()
    }

    pub fn release(&self) -> bool {
        self.options.release()
    }

    pub fn icons(&self) -> &Vec<PathBuf> {
        self.options.icons()
    }

    pub fn identifier(&self) -> &str {
        self.options.identifier()
    }

    pub fn profile(&self) -> String {
        if self.release() {
            "release".to_string()
        } else {
            "debug".to_string()
        }
    }

    pub fn version(&self) -> &Version {
        self.options.version()
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.options.vmmaker_vm()
    }

    pub fn libraries(&self) -> &Vec<Box<dyn Library>> {
        self.options.libraries()
    }

    pub fn app_name(&self) -> &str {
        self.options.app_name()
    }

    pub fn compilation_location(&self) -> PathBuf {
        self.target_dir()
            .join(self.target().to_string())
            .join(if self.release() { "release" } else { "debug" })
    }

    pub fn bundle_location(&self) -> PathBuf {
        self.options.bundle_dir().map_or_else(
            || self.default_bundle_location(),
            |bundle_dir| bundle_dir.to_path_buf(),
        )
    }

    /// A name of the corresponding executable in the bundle. The name either depends on the app name
    /// or on the executable name specified by the user
    pub fn bundled_executable_name(&self, executable: &Executable) -> String {
        executable.bundled_name(&self.options)
    }

    /// A name of the corresponding executable as compiled by cargo. The name either depends on the Cargo.toml of the vm-client
    pub fn compiled_executable_name(&self, executable: &Executable) -> String {
        executable.compiled_name(&self.options)
    }

    /// A path to the compiled binary after running cargo command.
    /// It is the same as defined in the [[bin]] section of the Cargo.toml
    pub fn compiled_executable_path(&self, executable: &Executable) -> PathBuf {
        self.compilation_location()
            .join(executable.compiled_name(&self.options))
    }

    pub fn default_bundle_location(&self) -> PathBuf {
        self.compilation_location().join("bundle")
    }

    pub fn third_party_libraries_directory(&self) -> PathBuf {
        self.options
            .workspace_directory()
            .unwrap_or(std::env::current_dir().unwrap())
            .join("third_party")
    }
}
