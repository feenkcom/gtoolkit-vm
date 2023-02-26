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
use std::ffi::{c_int, c_long, c_longlong, c_void, CString};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::builder::OsStr;
use clap::{arg, value_parser, Arg, ArgMatches, ColorChoice, Command, ValueEnum};

pub use runtime::*;
use vm_bindings::InterpreterConfiguration;

pub(crate) mod platform;
mod runtime;

pub fn main_cli() {
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

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    env::set_var("RUST_LOG", "error");

    std::thread::sleep(Duration::from_secs(1));
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Error),
    );

    let current_exe = env::current_exe().expect("Get current exe");
    let current_dir = env::current_dir().expect("Get current dir");
    let internal_path = app.internal_data_path().expect("Get internal data path");
    let external_path = app.external_data_path().expect("Get external data path");

    println!("current_exe: {}", current_exe.display());
    println!("current_dir: {}", current_dir.display());
    println!("internal_path: {}", internal_path.display());
    println!("external_path: {}", external_path.display());

    let image_path = external_path
        .join("glamoroustoolkit")
        .join("GlamorousToolkit.image");

    {
        let new_current_dir = image_path.parent().expect("Get parent directory");
        if !new_current_dir.exists() {
            panic!(".image directory does not exist");
        }
        env::set_current_dir(new_current_dir).unwrap_or_else(|error| {
            panic!(
                "Set current dir to {}: {}",
                new_current_dir.display(),
                error
            )
        });
    }

    println!("image_path exists: {:?}", image_path.exists());
    println!(
        "image_path metadata: {:?}",
        std::fs::metadata(&image_path).unwrap()
    );

    let mut extra_args = vec![];
    extra_args.push("--event-fetcher=winit".to_string());

    let mut configuration = InterpreterConfiguration::new(image_path);
    configuration.set_interactive_session(true);
    configuration.set_is_worker_thread(true);
    configuration.set_should_handle_errors(true);
    configuration.set_extra_arguments(extra_args);
    Constellation::for_android(app).run(configuration);
    std::thread::sleep(Duration::from_secs(1));
}
