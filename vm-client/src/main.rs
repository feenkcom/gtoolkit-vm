#![windows_subsystem = "windows"]

extern crate nfd2;
extern crate vm_bindings;

mod image_finder;
mod working_directory;

use crate::image_finder::{
    pick_image_with_dialog, search_image_file_nearby, validate_user_image_file,
};
use crate::working_directory::ensure_working_directory;
use clap::{AppSettings, ArgEnum, Clap};

use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use vm_bindings::{VMParameters, VM};

#[derive(Clap, Clone, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct AppOptions {
    /// A path to a custom Pharo .image to use instead of automatically detecting one
    #[clap(long, parse(from_os_str))]
    image: Option<PathBuf>,
}

impl AppOptions {
    pub fn image(&self) -> Option<PathBuf> {
        self.image.as_ref().map_or_else(
            || {
                search_image_file_nearby()
                    .map_or_else(|| pick_image_with_dialog(), |image| Some(image))
            },
            |image| Some(image.clone()),
        )
    }

    pub fn canonicalize(&mut self) {
        if let Some(ref image) = self.image {
            match fs::canonicalize(image) {
                Ok(image) => self.image = Some(image),
                Err(error) => {
                    panic!("Image does not exist {:?}: {:?}", image.display(), error)
                }
            }
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let executable_path =
        fs::canonicalize(Path::new(&args[0])).expect("Executable could not be located");

    let mut options: AppOptions = AppOptions::parse();
    options.canonicalize();

    ensure_working_directory();

    let image_path = match options.image() {
        None => {
            eprintln!("Could not find an .image file");
            return;
        }
        Some(path) => path,
    };

    let mut vm_args: Vec<String> = vec![];
    vm_args.push(executable_path.as_os_str().to_str().unwrap().to_owned());
    vm_args.push(image_path.as_os_str().to_str().unwrap().to_owned());

    let mut parameters = VMParameters::from_args(vm_args);
    parameters.set_image_file_name(image_path.as_os_str().to_str().unwrap().to_owned());
    parameters.set_is_interactive_session(true);

    VM::start(parameters).unwrap();
}
