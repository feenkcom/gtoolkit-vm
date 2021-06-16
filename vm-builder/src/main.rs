extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mustache;

mod bundlers;

use std::process::{Command, Stdio};

use clap::{AppSettings, ArgEnum, Clap};
use fs_extra::{copy_items, dir};
use std::str::FromStr;

use crate::bundlers::mac::MacBundler;
use crate::bundlers::Bundler;
use rustc_version::version_meta;
use std::path::PathBuf;

// Define your enum
#[derive(ArgEnum, Copy, Clone, Debug)]
#[repr(u32)]
enum Targets {
    #[clap(name = "x86_64-apple-darwin")]
    X8664appleDarwin,
}

impl FromStr for Targets {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Targets as ArgEnum>::from_str(s, true)
    }
}

impl ToString for Targets {
    fn to_string(&self) -> String {
        (Targets::VARIANTS[*self as usize]).to_owned()
    }
}

impl Targets {
    pub fn bundle_folder(&self) -> &'static str {
        match self {
            Targets::X8664appleDarwin => "osx",
        }
    }
}

#[derive(Clap, Clone, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct BuildOptions {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// To bundle a release build
    #[clap(long)]
    release: bool,
    #[clap(long, possible_values = Targets::VARIANTS, case_insensitive = true)]
    /// To cross-compile and bundle an application for another OS
    target: Option<Targets>,
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
    /// A list of icons of different sizes to package with the app. When packaging for MacOS the icons converted
    /// into one .icns icon file. If .icns file is provided it is used instead and not processed.
    #[clap(long)]
    icons: Option<Vec<String>>
}

const DEFAULT_BUILD_DIR: &str = "target";

fn main() {
    let build_config: BuildOptions = BuildOptions::parse();
    let final_config = compile_binary(&build_config);
    create_bundle(&final_config);
}

fn compile_binary(opts: &BuildOptions) -> BuildOptions {
    let mut final_config = opts.clone();

    let build_dir = opts
        .target_dir
        .as_ref()
        .map_or(DEFAULT_BUILD_DIR.to_owned(), |build_dir| build_dir.clone());

    let target = opts.target.as_ref().map_or_else(
        || <Targets as FromStr>::from_str(&*version_meta().unwrap().host).unwrap(),
        |target| target.clone(),
    );

    final_config.target_dir = Some(build_dir.clone());
    final_config.target = Some(target.clone());

    std::env::set_var("CARGO_TARGET_DIR", build_dir);

    let mut command = Command::new("cargo");
    command
        .stdout(Stdio::inherit())
        .arg("build")
        .arg("--package")
        .arg("vm-client")
        .arg("--target")
        .arg(target.to_string());

    match opts.verbose {
        0 => {}
        1 => {
            command.arg("-v");
        }
        _ => {
            command.arg("-vv");
        }
    }

    if opts.release {
        command.arg("--release");
    }

    command.status().unwrap();

    final_config
}

fn create_bundle(final_config: &BuildOptions) {
    let bundler = match final_config.target.as_ref().unwrap() {
        Targets::X8664appleDarwin => MacBundler::new(),
    };

    bundler.bundle(final_config);
}

fn package_libraries(final_config: &BuildOptions) {
    let mut bundle_dir = PathBuf::new();
    bundle_dir.push(final_config.target_dir.as_ref().unwrap());
    bundle_dir.push(final_config.target.as_ref().unwrap().to_string());
    bundle_dir.push(if final_config.release {
        "release"
    } else {
        "debug"
    });
    bundle_dir.push("bundle");
    bundle_dir.push(final_config.target.as_ref().unwrap().bundle_folder());

    let mut options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
    options.overwrite = true;
    let mut from_paths = Vec::new();

    from_paths.push(format!(
        "{}/Plugins",
        final_config.target_dir.as_ref().unwrap()
    ));

    copy_items(
        &from_paths,
        format!(
            "{}/GlamorousToolkit.app/Contents/MacOS/",
            bundle_dir.display()
        ),
        &options,
    )
    .unwrap();
}
