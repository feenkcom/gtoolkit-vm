use crate::{
    boxer, cairo, clipboard, freetype, git, gleam, glutin, pixman, sdl2, skia, winit, Library,
};
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

impl Target {
    pub fn is_unix(&self) -> bool {
        match self {
            Target::X8664appleDarwin => true,
            Target::AArch64appleDarwin => true,
            Target::X8664pcWindowsMsvc => false,
            Target::X8664UnknownlinuxGNU => true,
        }
    }

    pub fn is_windows(&self) -> bool {
        match self {
            Target::X8664appleDarwin => false,
            Target::AArch64appleDarwin => false,
            Target::X8664pcWindowsMsvc => true,
            Target::X8664UnknownlinuxGNU => false,
        }
    }
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
    #[clap(name = "sdl2")]
    Sdl2,
    #[clap(name = "boxer")]
    Boxer,
    #[clap(name = "pixman")]
    Pixman,
    #[clap(name = "freetype")]
    Freetype,
    #[clap(name = "cairo")]
    Cairo,
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
            ThirdPartyLibrary::Freetype => freetype().into(),
            ThirdPartyLibrary::Cairo => cairo().into(),
            ThirdPartyLibrary::Pixman => pixman().into(),
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
    /// Split build load over multiple threads
    #[clap(long)]
    multi_threaded: bool,
    #[clap(long, possible_values = Target::VARIANTS, case_insensitive = true)]
    /// To cross-compile and bundle an application for another OS
    target: Option<Target>,
    #[clap(long, parse(from_os_str))]
    /// Path to directory which cargo will use as the root of build directory.
    target_dir: Option<PathBuf>,
    /// A name of the app
    #[clap(long)]
    app_name: Option<String>,
    /// An output location of the bundle. By default a bundle is placed inside of the cargo's target dir in the following format: target/{target architecture}/{build|release}/
    #[clap(long, parse(from_os_str))]
    bundle_dir: Option<PathBuf>,
    /// MacOS only. Specify a path to a plist file to be bundled with the app
    #[clap(long, parse(from_os_str))]
    plist_file: Option<PathBuf>,
    /// Change the name of the executable. By default it is the same as app_name.
    #[clap(long)]
    executable_name: Option<String>,
    /// A future version in format X.Y.Z or vX.Y.Z
    #[clap(long)]
    version: Option<String>,
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
    /// Use a specific VM to run a VMMaker, must be a path to the executable. When specified, the build will not attempt to download a VM
    #[clap(long, parse(from_os_str))]
    vmmaker_vm: Option<PathBuf>,
}

impl BuildOptions {
    pub fn target(&self) -> Target {
        self.target.as_ref().map_or_else(
            || <Target as FromStr>::from_str(&*version_meta().unwrap().host).unwrap(),
            |target| target.clone(),
        )
    }

    pub fn target_dir(&self) -> Option<&Path> {
        self.target_dir.as_ref().map(|dir| dir.as_path())
    }

    pub fn bundle_dir(&self) -> Option<&Path> {
        self.bundle_dir.as_ref().map(|dir| dir.as_path())
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.vmmaker_vm.as_ref().map(|dir| dir.as_path())
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

    pub fn app_name(&self) -> Option<&str> {
        self.app_name.as_ref().map(|name| name.as_str())
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier
            .as_ref()
            .map(|identifier| identifier.as_str())
    }

    pub fn executable_name(&self) -> Option<&str> {
        self.executable_name.as_ref().map(|name| name.as_str())
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_ref().map(|version| version.as_str())
    }

    pub fn verbose(&self) -> i32 {
        self.verbose
    }

    pub fn release(&self) -> bool {
        self.release
    }

    pub fn multi_threaded(&self) -> bool {
        self.multi_threaded
    }

    pub fn icons(&self) -> Option<&Vec<String>> {
        self.icons.as_ref()
    }

    pub fn libraries(&self) -> Option<&Vec<ThirdPartyLibrary>> {
        self.libraries.as_ref()
    }
}
