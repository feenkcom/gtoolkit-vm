extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate cmake;
extern crate downloader;
extern crate feenk_releaser;
extern crate file_matcher;
extern crate flate2;
extern crate mustache;
extern crate pkg_config;
extern crate tar;
extern crate url;
extern crate user_error;
extern crate which;
extern crate xz2;

mod bundlers;
mod error;
mod libraries;
mod options;

pub use error::*;
pub use libraries::*;
pub use options::*;

use clap::Clap;

use crate::bundlers::linux::LinuxBundler;
use crate::bundlers::mac::MacBundler;
use crate::bundlers::windows::WindowsBundler;
use crate::bundlers::Bundler;
use crate::options::{BuildOptions, BundleOptions, Executable, Target};

fn main() -> Result<()> {
    let build_config: BuildOptions = BuildOptions::parse();

    let resolved_options = ResolvedOptions::new(build_config);
    let bundler = bundler(&resolved_options);

    let bundle_options =
        BundleOptions::new(resolved_options, vec![Executable::App, Executable::Cli]);

    bundler.ensure_third_party_requirements(&bundle_options);

    bundle_options.executables().iter().for_each(|executable| {
        let executable_options = ExecutableOptions::new(&bundle_options, executable.clone());
        bundler.pre_compile(&executable_options);
        bundler.compile_binary(&executable_options);
        bundler.post_compile(&executable_options)
    });

    bundler.compile_third_party_libraries(&bundle_options)?;
    bundler.bundle(&bundle_options);
    Ok(())
}

fn bundler(options: &ResolvedOptions) -> Box<dyn Bundler> {
    match options.target() {
        Target::X8664appleDarwin => Box::new(MacBundler::new()),
        Target::AArch64appleDarwin => Box::new(MacBundler::new()),
        Target::X8664pcWindowsMsvc => Box::new(WindowsBundler::new()),
        Target::X8664UnknownlinuxGNU => Box::new(LinuxBundler::new()),
    }
}
