use crate::{EventLoop, VirtualMachine};
use std::sync::Arc;
use vm_bindings::{NamedPrimitive, InterpreterParameters, LogLevel};

#[no_mangle]
pub static mut VIRTUAL_MACHINE: Option<Arc<VirtualMachine>> = None;
#[no_mangle]
pub fn has_virtual_machine() -> bool {
    unsafe { VIRTUAL_MACHINE.is_some() }
}

#[no_mangle]
pub fn primitiveGetAddressOfGToolkitVM() {
    println!("has VIRTUAL_MACHINE = {}", has_virtual_machine());
}

pub struct Constellation;
impl Constellation {
    pub fn run(parameters: InterpreterParameters) {
        let (event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(parameters, sender));
        unsafe { VIRTUAL_MACHINE = Some(vm.clone()) };

        vm.add_primitive(primitive!(primitiveGetAddressOfGToolkitVM));
        vm.start().unwrap();
    }
}
