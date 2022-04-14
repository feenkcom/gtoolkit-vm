use crate::{EventLoop, VirtualMachine};
use std::sync::Arc;
use vm_bindings::InterpreterConfiguration;

pub struct Constellation;
impl Constellation {
    pub fn run(configuration: InterpreterConfiguration) {
        if configuration.is_worker_thread() {
            Self::run_in_worker_thread(configuration);
        }
        else {
            Self::run_in_main_thread(configuration);
        }
    }

    fn run_in_main_thread(configuration: InterpreterConfiguration) {
        let vm = Arc::new(VirtualMachine::new(configuration, None, None));
        vm.clone().register();
        vm.start().unwrap();
    }

    fn run_in_worker_thread(configuration: InterpreterConfiguration) {
        let (mut event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(
            configuration,
            Some(event_loop),
            Some(sender),
        ));
        vm.clone().register();
        let join = vm.start().unwrap();
        vm.event_loop().unwrap().run().unwrap();
        join.unwrap().join().unwrap().unwrap();
    }
}
