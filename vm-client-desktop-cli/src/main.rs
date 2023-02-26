#![windows_subsystem = "console"]

#[macro_use]
extern crate log;

use std::env;

use clap::{arg, Arg, Command, value_parser, ValueEnum};
use clap::builder::PossibleValue;

use vm_runtime::{Constellation, print_short_version, print_version, validate_user_image_file};
use vm_runtime::vm_bindings::InterpreterConfiguration;

fn main() {
    env_logger::init();

    let app = Command::new("Virtual Machine")
        .author("feenk gmbh. <contact@feenk.com>")
        .allow_external_subcommands(true)
        .disable_version_flag(true)
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
                .action(clap::ArgAction::SetTrue)
                .help("Start image in the interactive (UI) mode"),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .action(clap::ArgAction::SetTrue)
                .help("Print the version information of the executable."),
        )
        .arg(
            Arg::new("short-version")
                .long("short-version")
                .action(clap::ArgAction::SetTrue)
                .help("Print just the version of the executable."),
        )
        .arg(
            arg!(<MODE>)
                .long("worker")
                .required(false)
                .value_parser(value_parser!(WorkerThreadMode))
                .help("Choose whether to run Pharo in a worker thread")
                .default_value(
                    WorkerThreadMode::Auto
                        .to_possible_value()
                        .unwrap()
                        .get_name()
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("no-error-handling")
                .long("no-error-handling")
                .action(clap::ArgAction::SetTrue)
                .help("Disable error handling by the virtual machine"),
        );

    let matches = app.get_matches();

    if matches.contains_id("version") {
        print_version();
        return;
    }
    if matches.contains_id("short-version") {
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
        env::set_current_dir(documents_dir).unwrap();
    }

    let image_path_string = matches
        .get_one::<String>("image")
        .map(|value| value.as_str());
    let current_dir = env::current_dir().unwrap();
    let image_path = match validate_user_image_file(image_path_string) {
        None => {
            eprintln!(
                "Could not find an .image file {:?}. Current directory: {}",
                &image_path_string,
                current_dir.display()
            );
            return;
        }
        Some(path) => path,
    };

    let mut extra_args: Vec<String> = vec![];
    if let Some((external, sub_m)) = matches.subcommand() {
        extra_args.push(external.to_owned());
        if let Some(values) = sub_m.get_many::<String>("") {
            for each in values {
                extra_args.push(each.to_owned());
            }
        }
    }

    let worker_mode = matches
        .get_one::<WorkerThreadMode>("MODE")
        .unwrap_or_else(|| &WorkerThreadMode::Auto);

    let mut configuration = InterpreterConfiguration::new(image_path);
    configuration.set_interactive_session(matches.contains_id("interactive"));
    configuration.set_is_worker_thread(worker_mode.should_run_in_worker_thread());
    configuration.set_should_handle_errors(!matches.contains_id("no-error-handling"));
    configuration.set_extra_arguments(extra_args);
    Constellation::new().run(configuration);
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WorkerThreadMode {
    /// Run the pharo interpreter in a worker thread freeing the main thread.
    Yes,
    /// Run the pharo interpreter on the main application thread.
    No,
    /// Automatically decide whether pharo interpreter should be run in a worker thread based on the current platform and support.
    Auto,
}

impl WorkerThreadMode {
    pub fn should_run_in_worker_thread(&self) -> bool {
        match self {
            WorkerThreadMode::Yes => true,
            WorkerThreadMode::No => false,
            WorkerThreadMode::Auto => cfg!(target_os = "macos") || cfg!(target_os = "windows"),
        }
    }


    pub fn possible_values() -> impl Iterator<Item = PossibleValue> {
        Self::value_variants()
            .iter()
            .filter_map(ValueEnum::to_possible_value)
    }
}