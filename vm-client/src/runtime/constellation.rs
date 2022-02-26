use crate::{EventLoop, VirtualMachine};
use std::sync::Arc;
use vm_bindings::InterpreterParameters;

pub struct Constellation;
impl Constellation {
    pub fn run(parameters: InterpreterParameters) {
        let vm = Arc::new(VirtualMachine::new(parameters, None, None));
        vm.clone().register();
        vm.start().unwrap();
    }

    pub fn run_worker(parameters: InterpreterParameters) {
        let (mut event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(
            parameters,
            Some(event_loop),
            Some(sender),
        ));
        vm.clone().register();
        let join = vm.start().unwrap();
        vm.event_loop().unwrap().run().unwrap();
        join.unwrap().join().unwrap().unwrap();
    }
}
