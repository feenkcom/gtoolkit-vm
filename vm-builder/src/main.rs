extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate cmake;
extern crate mustache;
extern crate url;
extern crate user_error;
extern crate which;

mod bundlers;
mod libraries;
mod options;

pub use libraries::*;

use clap::Clap;

use std::process::Command;

use crate::bundlers::linux::LinuxBundler;
use crate::bundlers::mac::MacBundler;
use crate::bundlers::windows::WindowsBundler;
use crate::bundlers::Bundler;
use crate::options::{BuildOptions, FinalOptions, Target};

fn main() {
    let build_config: BuildOptions = BuildOptions::parse();
    let final_config = FinalOptions::new(build_config);

    let bundler = bundler(&final_config);
    bundler.ensure_third_party_requirements(&final_config);
    bundler.pre_compile(&final_config);
    compile_binary(&final_config);
    bundler.post_compile(&final_config);
    bundler.compile_third_party_libraries(&final_config);
    bundler.bundle(&final_config);
}

fn compile_binary(opts: &FinalOptions) {
    std::env::set_var("CARGO_TARGET_DIR", opts.target_dir());

    std::env::set_var(
        "VM_CLIENT_EMBED_DEBUG_SYMBOLS",
        format!("{}", opts.debug_symbols()),
    );

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--package")
        .arg("vm-client")
        .arg("--target")
        .arg(opts.target().to_string());

    match opts.verbose() {
        0 => {}
        1 => {
            command.arg("-v");
        }
        _ => {
            command.arg("-vv");
        }
    }

    if opts.release() {
        command.arg("--release");
    }

    if !command.status().unwrap().success() {
        panic!("Failed to compile a vm-client")
    }
}

fn bundler(final_config: &FinalOptions) -> Box<dyn Bundler> {
    match final_config.target() {
        Target::X8664appleDarwin => Box::new(MacBundler::new()),
        Target::AArch64appleDarwin => Box::new(MacBundler::new()),
        Target::X8664pcWindowsMsvc => Box::new(WindowsBundler::new()),
        Target::X8664UnknownlinuxGNU => Box::new(LinuxBundler::new()),
    }
}
