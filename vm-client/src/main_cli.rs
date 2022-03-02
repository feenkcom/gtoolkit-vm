#![windows_subsystem = "console"]
#[macro_use]
extern crate vm_bindings;
extern crate num;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

pub(crate) mod platform;
mod runtime;
pub use runtime::*;

use clap::{App, AppSettings, Arg};
use std::sync::mpsc::channel;
use std::sync::Arc;
use vm_bindings::InterpreterParameters;

fn main() {
    let matches = App::new("Virtual Machine")
        .version("1.0")
        .author("feenk gmbh. <contact@feenk.com>")
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(
            Arg::new("image")
                .value_name("image")
                .index(1)
                .required(true)
                .help("A path to an image file to run"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .help("Start image in the interactive (UI) mode"),
        )
        .arg(
            Arg::new("worker")
                .long("worker")
                .help("Start image in the worker thread"),
        )
        .get_matches();

    let image_path = match validate_user_image_file(matches.value_of("image")) {
        None => {
            eprintln!("Could not find an .image file");
            return;
        }
        Some(path) => path,
    };

    let mut vm_args: Vec<String> = vec![];
    vm_args.push(std::env::args().collect::<Vec<String>>()[0].to_owned());
    vm_args.push(image_path.as_os_str().to_str().unwrap().to_owned());

    if let Some((external, sub_m)) = matches.subcommand() {
        vm_args.push(external.to_owned());
        if let Some(values) = sub_m.values_of("") {
            for each in values {
                vm_args.push(each.to_owned());
            }
        }
    }

    let mut parameters = InterpreterParameters::from_args(vm_args);
    parameters.set_image_file_name(image_path.as_os_str().to_str().unwrap().to_owned());
    parameters.set_is_interactive_session(matches.is_present("interactive"));

    if matches.is_present("worker") {
        Constellation::run_worker(parameters);
    }
    else {
        Constellation::run(parameters);
    }
}
