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

pub use crate::runtime::*;

pub(crate) mod platform;
mod runtime;

fn main() {
    vm_client::main_cli()
}
