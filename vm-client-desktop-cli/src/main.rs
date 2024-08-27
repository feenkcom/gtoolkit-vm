#![windows_subsystem = "console"]

#[macro_use]
extern crate log;

use std::env;
use std::ffi::OsString;

use clap::builder::PossibleValue;
use clap::{arg, value_parser, Arg, Command, ValueEnum};

use vm_runtime::vm_bindings::InterpreterConfiguration;
use vm_runtime::{
    print_short_version, print_version, validate_user_image_file, Constellation,
    VirtualMachineConfiguration,
};

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
            Arg::new("beacon")
                .long("beacon")
                .action(clap::ArgAction::Append)
                .conflicts_with("beacon-all")
                .help("Enable Beacon VM signals to be logged to the console"),
        )
        .arg(
            Arg::new("beacon-all")
                .long("beacon-all")
                .action(clap::ArgAction::SetTrue)
                .help("Enable logging of all Beacon signals to the console"),
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
            Arg::new("print-stack-on-signals")
                .long("print-stack-on-signals")
                .action(clap::ArgAction::SetTrue)
                .help(
                    "Enable virtual machine stack printing when encountering OS platform signals",
                ),
        )
         .arg(
            Arg::new("should-avoid-searching-segments-with-pinned-objects")
                .long("should-avoid-searching-segments-with-pinned-objects")
                .action(clap::ArgAction::SetTrue)
                .help(
                    "Pablos questionable command line parameter",
                ),
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
        );

    #[cfg(target_os = "linux")]
    let app = app.arg(
        Arg::new("enable-wayland")
            .long("enable-wayland")
            .action(clap::ArgAction::SetTrue)
            .help("Enable wayland support. In the future, this may be enabled by default"),
    );

    let matches = app.get_matches();

    if matches.get_flag("version") {
        print_version();
        return;
    }
    if matches.get_flag("short-version") {
        print_short_version();
        return;
    }

    #[cfg(target_os = "linux")]
    {
        if !matches.get_flag("enable-wayland") {
            env::remove_var("WAYLAND_DISPLAY");
        }
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
        if let Some(values) = sub_m.get_many::<OsString>("") {
            for each in values {
                extra_args.push(each.to_str().unwrap().to_string());
            }
        }
    }

    let worker_mode = matches
        .get_one::<WorkerThreadMode>("MODE")
        .unwrap_or_else(|| &WorkerThreadMode::Auto);

    let mut interpreter_configuration = InterpreterConfiguration::new(image_path);
    interpreter_configuration.set_interactive_session(matches.get_flag("interactive"));
    interpreter_configuration.set_should_avoid_searching_segments_with_pinned_objects(matches.get_flag("should-avoid-searching-segments-with-pinned-objects"));
    interpreter_configuration.set_is_worker_thread(worker_mode.should_run_in_worker_thread());
    interpreter_configuration
        .set_should_print_stack_on_signals(matches.get_flag("print-stack-on-signals"));
    interpreter_configuration.set_extra_arguments(extra_args);

    let log_signals = matches
        .get_many::<String>("beacon")
        .map(|values| {
            let mut signals = values
                .map(|each| each.split_whitespace())
                .flatten()
                .map(|each| each.to_string())
                .collect::<Vec<String>>();
            signals.dedup();
            signals
        })
        .or_else(|| {
            if matches.get_flag("beacon-all") {
                Some(vec![])
            } else {
                None
            }
        });

    Constellation::new().run(VirtualMachineConfiguration {
        interpreter_configuration,
        log_signals,
    });
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
            WorkerThreadMode::Auto => {
                cfg!(target_os = "macos")
                    || cfg!(target_os = "windows")
                    || cfg!(target_os = "linux")
                    || cfg!(target_os = "android")
            }
        }
    }

    pub fn possible_values() -> impl Iterator<Item = PossibleValue> {
        Self::value_variants()
            .iter()
            .filter_map(ValueEnum::to_possible_value)
    }
}
