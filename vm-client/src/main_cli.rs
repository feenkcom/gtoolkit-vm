#![windows_subsystem = "console"]
extern crate vm_bindings;

mod image_finder;

use clap::{App, AppSettings, Arg, ArgMatches};
use image_finder::{search_image_file_nearby, validate_user_image_file};
use std::fs;
use std::path::PathBuf;
use vm_bindings::{VMParameters, VM};

fn main() {
    let matches = App::new("Virtual Machine")
        .version("1.0")
        .author("feenk gmbh. <contact@feenk.com>")
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::new("image")
                .value_name("image")
                .index(1)
                .required(true)
                .about("A path to an image file to run"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .about("Start image in the interactive (UI) mode"),
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

    match matches.subcommand() {
        Some((external, sub_m)) => {
            vm_args.push(external.to_owned());
            for each in sub_m.values_of("").unwrap() {
                vm_args.push(each.to_owned());
            }
        }
        _ => {}
    }

    let mut parameters = VMParameters::from_args(vm_args);
    parameters.set_image_file_name(image_path.as_os_str().to_str().unwrap().to_owned());
    parameters.set_is_interactive_session(matches.is_present("interactive"));

    VM::start(parameters).unwrap();
}
