use crate::{EventLoop, VirtualMachine};
use std::sync::Arc;
use vm_bindings::InterpreterParameters;

pub struct Constellation;
impl Constellation {
    pub fn run(parameters: InterpreterParameters) {
        let vm = Arc::new(VirtualMachine::new(parameters, None));
        vm.clone().register();
        vm.start().unwrap();
    }

    pub fn run_worker(parameters: InterpreterParameters) {
        let (event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(parameters, Some(sender)));
        vm.clone().register();
        let join = vm.start().unwrap();

        event_loop.run().unwrap();
        join.unwrap().join().unwrap().unwrap();
    }
}
