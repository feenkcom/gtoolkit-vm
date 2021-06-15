extern crate clap;

use std::process::{Command, Stdio};

use clap::{AppSettings, Clap};
use fs_extra::{copy_items, dir};
use std::str::FromStr;

use clap::ArgEnum;
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
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// To bundle a release build
    #[clap(short, long)]
    release: bool,
    #[clap(short, long, possible_values = Targets::VARIANTS, case_insensitive = true)]
    /// To cross-compile and bundle an application for another OS
    target: Option<Targets>,
    #[clap(short, long)]
    /// Path to directory which cargo will use as the root of build directory.
    build: Option<String>,
}

const DEFAULT_BUILD_DIR: &str = "target";

fn main() {
    let build_config: Opts = Opts::parse();
    let final_config = compile_bundle(&build_config);
    package_libraries(&final_config);
}

fn compile_bundle(opts: &Opts) -> Opts {
    let mut final_config = opts.clone();

    let build_dir = opts
        .build
        .as_ref()
        .map_or(DEFAULT_BUILD_DIR.to_owned(), |build_dir| build_dir.clone());

    let target = opts.target.as_ref().map_or_else(
        || <Targets as FromStr>::from_str(&*version_meta().unwrap().host).unwrap(),
        |target| target.clone(),
    );

    final_config.build = Some(build_dir.clone());
    final_config.target = Some(target.clone());

    std::env::set_var("CARGO_TARGET_DIR", format!("../{}", build_dir));

    let mut command = Command::new("cargo");
    command
        .stdout(Stdio::inherit())
        .current_dir("vm-client")
        .arg("bundle")
        .arg("--target")
        .arg(target.to_string());

    if opts.release {
        command.arg("--release");
    }

    command.status().unwrap();

    final_config
}

fn package_libraries(final_config: &Opts) {
    let mut bundle_dir = PathBuf::new();
    bundle_dir.push(final_config.build.as_ref().unwrap());
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

    from_paths.push(format!("{}/Plugins", final_config.build.as_ref().unwrap()));

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
