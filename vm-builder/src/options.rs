use crate::{boxer, clipboard, git, gleam, glutin, sdl2, skia, winit, Library};
use clap::{AppSettings, ArgEnum, Clap};
use rustc_version::version_meta;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

#[derive(ArgEnum, Copy, Clone, Debug)]
#[repr(u32)]
pub enum Target {
    #[clap(name = "x86_64-apple-darwin")]
    X8664appleDarwin,
    #[clap(name = "aarch64-apple-darwin")]
    AArch64appleDarwin,
    #[clap(name = "x86_64-pc-windows-msvc")]
    X8664pcWindowsMsvc,
    #[clap(name = "x86_64-unknown-linux-gnu")]
    X8664UnknownlinuxGNU,
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Target as ArgEnum>::from_str(s, true)
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        (Target::VARIANTS[*self as usize]).to_owned()
    }
}

#[derive(ArgEnum, Copy, Clone, Debug)]
#[repr(u32)]
pub enum ThirdPartyLibrary {
    #[clap(name = "git")]
    Git,
    #[clap(name = "sld2")]
    Sdl2,
    #[clap(name = "boxer")]
    Boxer,
    #[clap(name = "skia")]
    Skia,
    #[clap(name = "glutin")]
    Glutin,
    #[clap(name = "gleam")]
    Gleam,
    #[clap(name = "winit")]
    Winit,
    #[clap(name = "clipboard")]
    Clipboard,
}

impl FromStr for ThirdPartyLibrary {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <ThirdPartyLibrary as ArgEnum>::from_str(s, true)
    }
}

impl ToString for ThirdPartyLibrary {
    fn to_string(&self) -> String {
        (ThirdPartyLibrary::VARIANTS[*self as usize]).to_owned()
    }
}

impl ThirdPartyLibrary {
    pub fn as_library(&self) -> Box<dyn Library> {
        match self {
            ThirdPartyLibrary::Boxer => boxer().into(),
            ThirdPartyLibrary::Skia => skia().into(),
            ThirdPartyLibrary::Glutin => glutin().into(),
            ThirdPartyLibrary::Gleam => gleam().into(),
            ThirdPartyLibrary::Winit => winit().into(),
            ThirdPartyLibrary::Clipboard => clipboard().into(),
            ThirdPartyLibrary::Git => git().into(),
            ThirdPartyLibrary::Sdl2 => sdl2().into(),
        }
    }
}

#[derive(Clap, Clone, Debug, Default)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct BuildOptions {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// To bundle a release build
    #[clap(long)]
    release: bool,
    /// Include debug symbols in the binary
    #[clap(long)]
    debug_symbols: bool,
    #[clap(long, possible_values = Target::VARIANTS, case_insensitive = true)]
    /// To cross-compile and bundle an application for another OS
    target: Option<Target>,
    #[clap(long)]
    /// Path to directory which cargo will use as the root of build directory.
    target_dir: Option<String>,
    /// A name of the app
    #[clap(long)]
    app_name: Option<String>,
    /// An output location of the bundle. By default a bundle is placed inside of the cargo's target dir in the following format: target/{target architecture}/{build|release}/
    #[clap(long)]
    bundle_dir: Option<String>,
    /// MacOS only. Specify a path to a plist file to be bundled with the app
    #[clap(long)]
    plist_file: Option<String>,
    /// Change the name of the executable. By default it is the same as app_name.
    #[clap(long)]
    executable_name: Option<String>,
    /// A major version of the bundle. Defaults to 0 if minor is specified.
    #[clap(long)]
    major_version: Option<usize>,
    /// A minor version of the bundle. Defaults to 0 if major is specified.
    #[clap(long)]
    minor_version: Option<usize>,
    /// A patch version of the bundle. Defaults to 0.
    #[clap(long)]
    patch_version: Option<usize>,
    /// A unique app identifier in the reverse domain notation, for example com.example.app
    #[clap(long)]
    identifier: Option<String>,
    /// An author entity of the application (company or person)
    #[clap(long)]
    author: Option<String>,
    /// A list of icons of different sizes to package with the app. When packaging for MacOS the icons converted
    /// into one .icns icon file. If .icns file is provided it is used instead and not processed.
    #[clap(long)]
    icons: Option<Vec<String>>,
    #[clap(long, possible_values = ThirdPartyLibrary::VARIANTS, case_insensitive = true)]
    /// Include third party libraries
    libraries: Option<Vec<ThirdPartyLibrary>>,
}

const DEFAULT_BUILD_DIR: &str = "target";
#[derive(Debug)]
pub struct FinalOptions {
    build_options: BuildOptions,
}

impl FinalOptions {
    pub fn new(build_options: BuildOptions) -> Self {
        let mut final_config = Self { build_options };

        let build_dir = final_config.build_options.target_dir.as_ref().map_or_else(
            || {
                final_config.workspace_directory().map_or(
                    DEFAULT_BUILD_DIR.to_owned(),
                    |workspace| {
                        workspace
                            .join(DEFAULT_BUILD_DIR)
                            .to_str()
                            .unwrap()
                            .to_owned()
                    },
                )
            },
            |build_dir| build_dir.clone(),
        );

        let target = final_config.build_options.target.as_ref().map_or_else(
            || <Target as FromStr>::from_str(&*version_meta().unwrap().host).unwrap(),
            |target| target.clone(),
        );

        final_config.build_options.target_dir = Some(build_dir.clone());
        final_config.build_options.target = Some(target.clone());

        final_config
    }

    pub fn target(&self) -> Target {
        self.build_options.target.unwrap()
    }

    pub fn target_dir(&self) -> PathBuf {
        Path::new(self.build_options.target_dir.as_ref().unwrap()).to_path_buf()
    }

    pub fn debug_symbols(&self) -> bool {
        self.build_options.debug_symbols
    }

    pub fn verbose(&self) -> i32 {
        self.build_options.verbose
    }

    pub fn release(&self) -> bool {
        self.build_options.release
    }

    pub fn icons(&self) -> Vec<String> {
        self.build_options
            .icons
            .as_ref()
            .map_or(vec![], |icons| icons.clone())
    }

    pub fn identifier(&self) -> Option<String> {
        self.build_options.identifier.clone()
    }

    pub fn major_version(&self) -> usize {
        self.build_options.major_version.unwrap_or_else(|| {
            if self.build_options.minor_version.is_some()
                | self.build_options.patch_version.is_some()
            {
                0
            } else {
                1
            }
        })
    }

    pub fn profile(&self) -> String {
        if self.release() {
            "release".to_string()
        } else {
            "debug".to_string()
        }
    }

    pub fn minor_version(&self) -> usize {
        self.build_options.minor_version.unwrap_or(0)
    }

    pub fn patch_version(&self) -> usize {
        self.build_options.patch_version.unwrap_or(0)
    }

    pub fn bundle_dir(&self) -> Option<String> {
        self.build_options.bundle_dir.clone()
    }

    pub fn workspace_directory(&self) -> Option<PathBuf> {
        let output = Command::new("cargo")
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format")
            .arg("plain")
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute command");

        let workspace_toml_path =
            PathBuf::new().join(String::from_utf8_lossy(&output.stdout).to_string());
        Some(workspace_toml_path.parent().unwrap().to_path_buf())
    }

    pub fn third_party_libraries(&self) -> Vec<Box<dyn Library>> {
        self.build_options
            .libraries
            .as_ref()
            .map_or(vec![], |libraries| {
                libraries.iter().map(|each| each.as_library()).collect()
            })
    }

    pub fn app_name(&self) -> String {
        self.build_options
            .app_name
            .as_ref()
            .map_or("VM".to_owned(), |name| name.to_owned())
    }

    pub fn executable_name(&self) -> String {
        let mut executable_name = self
            .build_options
            .executable_name
            .as_ref()
            .map_or_else(|| self.app_name(), |name| name.to_owned());

        if let Some(extension) = self.executable_extension() {
            executable_name = format!("{}.{}", &executable_name, extension);
        };
        executable_name
    }

    pub fn cli_executable_name(&self) -> String {
        let mut executable_name = self
            .build_options
            .executable_name
            .as_ref()
            .map_or_else(|| self.app_name(), |name| name.to_owned());

        executable_name = format!("{}-cli", executable_name);

        if let Some(extension) = self.executable_extension() {
            executable_name = format!("{}.{}", &executable_name, extension);
        } else {
        }
        executable_name
    }

    pub fn executable_extension(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        return None;
        #[cfg(target_os = "macos")]
        return None;
        #[cfg(target_os = "windows")]
        return Some("exe".to_string());
    }

    pub fn compilation_location(&self) -> PathBuf {
        self.target_dir()
            .join(self.target().to_string())
            .join(if self.release() { "release" } else { "debug" })
    }

    pub fn bundle_location(&self) -> PathBuf {
        self.bundle_dir().map_or_else(
            || self.default_bundle_location(),
            |bundle_dir| PathBuf::new().join(&bundle_dir),
        )
    }

    pub fn default_bundle_location(&self) -> PathBuf {
        self.compilation_location().join("bundle")
    }

    pub fn third_party_libraries_directory(&self) -> PathBuf {
        self.workspace_directory()
            .unwrap_or(std::env::current_dir().unwrap())
            .join("third_party")
    }
}
