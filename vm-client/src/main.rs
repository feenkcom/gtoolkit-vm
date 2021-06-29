#![windows_subsystem = "windows"]

extern crate nfd2;
extern crate vm_bindings;

use clap::{App, AppSettings, Arg, ArgMatches};
use nfd2::{dialog, Response};
use std::fs;
use std::path::PathBuf;
use vm_bindings::{VMParameters, VM};

fn try_find_image_file_in_directory(path: PathBuf) -> Option<PathBuf> {
    let files = fs::read_dir(&path).unwrap();
    let image_files: Vec<PathBuf> = files
        .filter_map(Result::ok)
        .filter(|d| {
            if let Some(e) = d.path().extension() {
                e == "image"
            } else {
                false
            }
        })
        .map(|d| d.path().to_path_buf())
        .collect();

    match image_files.len() {
        1 => Some(image_files[0].clone()),
        _ => None,
    }
}

fn search_image_file_nearby() -> Option<PathBuf> {
    let image_file = std::env::current_exe().map_or(None, |path| {
        path.parent().map_or(None, |exe_path| {
            try_find_image_file_in_directory(exe_path.to_path_buf())
        })
    });

    if image_file.is_some() {
        return image_file;
    }

    std::env::current_dir().map_or(None, |path| try_find_image_file_in_directory(path))
}

fn validate_user_image_file(image_name: Option<&str>) -> Option<PathBuf> {
    if let Some(image_file_name) = image_name {
        let image_path = PathBuf::new().join(image_file_name);
        if image_path.exists() {
            return Some(image_path);
        }
    }
    None
}

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
    // if true, we will attempt to find an .image file automatically and if not found show a file picker dialog
    // if false, an image file must be specified, unless --image-picker flag is set
    let wants_interactive =
        std::env::var("WANTS_INTERACTIVE_SESSION").map_or(false, |value| value == "true");

    if wants_interactive {
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
    }

    let mut image_argument = Arg::new("image")
        .value_name("image")
        .index(1)
        .about("A path to an image file to run");

    if !wants_interactive {
        image_argument = image_argument.required_unless_present("image-picker")
    }

    let matches = App::new("Virtual Machine")
        .version("1.0")
        .author("feenk gmbh. <contact@feenk.com>")
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::ColoredHelp)
        .arg(image_argument)
        .arg(
            Arg::new("image-picker")
                .long("image-picker")
                .takes_value(false)
                .long_about("Use interactive image picker. First try to find an .image file in the same folder as executable and in the current workspace directory, if no image was found show an .image file picker dialog"),
        )
        .get_matches();

    // If evaluates to true, we should try to find an image in the nearby folder or show an image picker dialog if image was not specified
    let should_use_image_picker = wants_interactive | matches.is_present("image-picker");

    if !matches.is_present("image") & !should_use_image_picker {
        eprintln!(".image is not specified");
        return;
    }

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
