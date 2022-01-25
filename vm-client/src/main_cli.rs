#![windows_subsystem = "console"]
extern crate vm_bindings;

mod error;
mod event_loop;
mod image_finder;
mod virtual_machine;

use clap::{App, AppSettings, Arg};
use image_finder::validate_user_image_file;
use std::sync::mpsc::channel;
use std::sync::Arc;
use vm_bindings::InterpreterParameters;

pub use event_loop::{EventLoop, EventLoopMessage};
pub use virtual_machine::VirtualMachine;

#[no_mangle]
pub static mut VIRTUAL_MACHINE: Option<Arc<VirtualMachine>> = None;
#[no_mangle]
pub fn has_virtual_machine() -> bool {
    unsafe { VIRTUAL_MACHINE.is_some() }
}

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
                .help("Run vm in the worker process"),
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
    if matches.is_present("worker") {
        vm_args.push("--worker".to_string());
    }
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

    println!("{:?}", &parameters);
    run(parameters);
}

fn run(parameters: InterpreterParameters) {
    let (event_loop, sender) = EventLoop::new();

    let vm = Arc::new(VirtualMachine::new(parameters, sender));
    unsafe { VIRTUAL_MACHINE = Some(vm.clone()) };
    let join = vm.start().unwrap();

    //join.is_running()

    let result = join.join().unwrap();
    result.unwrap();
}
