#![windows_subsystem = "console"]

#[macro_use]
extern crate default_env;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_traits;
#[macro_use]
extern crate vm_bindings;

use std::env;
use std::env::Args;
use std::path::Path;
use clap::{arg, App, AppSettings, Arg, ArgEnum};

pub use runtime::*;
use vm_bindings::InterpreterConfiguration;

pub(crate) mod platform;
mod runtime;

fn main() {
    env_logger::init();

    let matches = App::new("Virtual Machine")
        .author("feenk gmbh. <contact@feenk.com>")
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::NoAutoVersion)
        .arg(
            Arg::new("image")
                .value_name("image")
                .index(1)
                .required_unless_present("version")
                .required_unless_present("short-version")
                .help("A path to an image file to run"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .help("Start image in the interactive (UI) mode"),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .help("Print the version information of the executable."),
        )
        .arg(
            Arg::new("short-version")
                .long("short-version")
                .help("Print just the version of the executable."),
        )
        .arg(
            arg!(<MODE>)
                .long("worker")
                .required(false)
                .default_value(
                    WorkerThreadMode::Auto
                        .to_possible_value()
                        .unwrap()
                        .get_name(),
                )
                .long_help(WorkerThreadMode::long_help_str())
                .possible_values(WorkerThreadMode::possible_values()),
        )
        .arg(
            Arg::new("no-error-handling")
                .long("no-error-handling")
                .help("Disable error handling by the virtual machine"),
        )
        .get_matches();

    if matches.is_present("version") {
        print_version();
        return;
    }
    if matches.is_present("short-version") {
        print_short_version();
        return;
    }

    // iOS sandboxes applications and does not allow developers
    // to write inside of the application folder.
    // In addition, the app is executed with `/` as the current_dir.
    #[cfg(target_os = "ios")]
    {
        let home_dir = pathos::user::home_dir().unwrap();
        let documents_dir = home_dir.join("Documents");
        std::env::set_current_dir(documents_dir).unwrap();
    }

    let image_path_string = matches.value_of("image");
    let current_dir = std::env::current_dir().unwrap();
    let image_path = match validate_user_image_file(image_path_string) {
        None => {
            eprintln!(
                "Could not find an .image file {:?} in {}",
                &image_path_string,
                current_dir.display()
            );
            return;
        }
        Some(path) => current_dir.join(path),
    };

    let mut extra_args: Vec<String> = vec![];
    if let Some((external, sub_m)) = matches.subcommand() {
        extra_args.push(external.to_owned());
        if let Some(values) = sub_m.values_of("") {
            for each in values {
                extra_args.push(each.to_owned());
            }
        }
    }

    let worker_mode = matches
        .value_of_t("MODE")
        .unwrap_or_else(|_| WorkerThreadMode::Auto);

    let mut configuration = InterpreterConfiguration::new(image_path);
    configuration.set_interactive_session(matches.is_present("interactive"));
    configuration.set_is_worker_thread(worker_mode.should_run_in_worker_thread());
    configuration.set_should_handle_errors(!matches.is_present("no-error-handling"));
    configuration.set_extra_arguments(extra_args);
    Constellation::run(configuration);
}
