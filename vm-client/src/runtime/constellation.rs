use crate::{EventLoop, VirtualMachine};
use std::sync::Arc;
use vm_bindings::InterpreterParameters;

pub struct Constellation;
impl Constellation {
    pub fn run(parameters: InterpreterParameters) {
        let (event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(parameters, sender));
        vm.clone().register();
        vm.start().unwrap();
    }
}
