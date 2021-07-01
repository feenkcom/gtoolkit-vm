#![windows_subsystem = "windows"]

extern crate nfd2;
extern crate vm_bindings;

mod image_finder;

use crate::image_finder::{search_image_file_nearby, validate_user_image_file};
use clap::{App, AppSettings, Arg, ArgMatches};
use nfd2::{dialog, Response};
use std::fs;
use std::path::PathBuf;
use vm_bindings::{VMParameters, VM};

fn pick_image_with_dialog() -> Option<PathBuf> {
    let result = dialog().filter("image").open().unwrap_or_else(|e| {
        panic!("{}", e);
    });

    match result {
        Response::Okay(file_name) => {
            let file_path = PathBuf::new().join(file_name);
            if file_path.exists() {
                Some(file_path)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn detect_image(matches: &ArgMatches) -> Option<PathBuf> {
    if matches.is_present("image") {
        return validate_user_image_file(matches.value_of("image"));
    }

    if let Some(image) = search_image_file_nearby() {
        return Some(image);
    }
    pick_image_with_dialog()
}

fn main() {
    let app_dir = std::env::current_exe().map_or(None, |exe_path| {
        exe_path.parent().map_or(None, |parent| {
            parent.parent().map_or(None, |parent| {
                parent.parent().map_or(None, |parent| {
                    parent
                        .parent()
                        .map_or(None, |parent| Some(parent.to_path_buf()))
                })
            })
        })
    });
    if app_dir.is_some() {
        std::env::set_current_dir(app_dir.unwrap()).unwrap();
    }

    let matches = App::new("Virtual Machine")
        .version("1.0")
        .author("feenk gmbh. <contact@feenk.com>")
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::new("image")
                .value_name("image")
                .index(1)
                .about("A path to an image file to run"),
        )
        .get_matches();

    let image_path = match detect_image(&matches) {
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
    parameters.set_is_interactive_session(true);

    VM::start(parameters).unwrap();
}
