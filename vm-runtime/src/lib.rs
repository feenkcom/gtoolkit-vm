#[macro_use]
pub extern crate vm_bindings;
#[macro_use]
extern crate default_env;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(target_os = "android")]
pub extern crate android_activity;

mod constellation;
mod error;
mod event_loop;
#[cfg(feature = "ffi")]
mod ffi;
mod image_finder;
mod logger;
mod version;
mod virtual_machine;
mod working_directory;

pub mod objects;
#[cfg(feature = "pharo-compiler")]
mod pharo_compiler;
mod reference_finder;
mod telemetry;

pub use constellation::Constellation;
pub use error::{ApplicationError, Result};
pub use event_loop::{EventLoop, EventLoopMessage, EventLoopWaker};
#[cfg(feature = "ffi")]
pub use ffi::{primitiveEventLoopCallout, primitiveExtractReturnValue, EventLoopCallout};
pub use image_finder::*;
pub use logger::*;
pub use telemetry::*;
pub use version::{fetch_version, print_short_version, print_version};
pub use virtual_machine::{vm, VirtualMachine, VirtualMachineConfiguration};
pub use working_directory::executable_working_directory;
