#[macro_use]
extern crate num_derive;

mod bindings;
mod export;
mod interpreter;
mod interpreter_config;
mod interpreter_marshalling;
mod interpreter_proxy;
mod parameter_vector;
mod parameters;
mod prelude;

pub use export::NamedPrimitive;
pub use interpreter::{LogLevel, PharoInterpreter};
pub use interpreter_config::InterpreterConfiguration;
pub use interpreter_marshalling::Marshallable;
pub use interpreter_proxy::{InterpreterProxy, ObjectFieldIndex, ObjectPointer, StackOffset};

// re-export ffi
#[cfg(feature = "libffi")]
pub use libffi;
#[cfg(feature = "libffi-sys")]
pub use libffi_sys;

pub fn virtual_machine_info() -> &'static str {
    include_str!(env!("VM_INFO"))
}
