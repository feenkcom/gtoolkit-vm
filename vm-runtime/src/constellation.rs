use crate::{EventLoop, VirtualMachine, VirtualMachineConfiguration};
use std::sync::Arc;

#[derive(Debug)]
pub struct Constellation {
    #[cfg(target_os = "android")]
    android_app: Option<android_activity::AndroidApp>,
}

impl Constellation {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "android")]
            android_app: None,
        }
    }

    #[cfg(target_os = "android")]
    pub fn for_android(android_app: android_activity::AndroidApp) -> Self {
        Self {
            android_app: Some(android_app),
        }
    }

    pub fn run(self, configuration: VirtualMachineConfiguration) {
        if configuration.interpreter_configuration.is_worker_thread() {
            self.run_in_worker_thread(configuration);
        } else {
            self.run_in_main_thread(configuration);
        }
    }

    fn run_in_main_thread(self, configuration: VirtualMachineConfiguration) {
        let vm = Arc::new(VirtualMachine::new(
            configuration,
            None,
            None,
            #[cfg(target_os = "android")]
            self.android_app.expect("AndroidApp must be initialized"),
        ));
        vm.clone().register();
        vm.start().unwrap();
    }

    fn run_in_worker_thread(self, configuration: VirtualMachineConfiguration) {
        let (event_loop, sender) = EventLoop::new();

        let vm = Arc::new(VirtualMachine::new(
            configuration,
            Some(event_loop),
            Some(sender),
            #[cfg(target_os = "android")]
            self.android_app.expect("AndroidApp must be initialized"),
        ));
        vm.clone().register();
        let join = vm.start().unwrap();
        vm.event_loop().unwrap().run().unwrap();
        join.unwrap().join().unwrap().unwrap();
    }
}
