use crate::{vm, LogSignal, Logger, NullLogger, VM_LOGGER};
use num_traits::FromPrimitive;
use std::any::Any;
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::mem::size_of;
pub use std::os::raw::{c_char, c_int};
use vm_bindings::{LogLevel, ObjectFieldIndex, StackOffset};

use std::sync::Mutex;

#[derive(Debug)]
struct BeaconLogger {
    semaphore: usize,
    buffered_logs: Vec<LogSignal>,
}

impl BeaconLogger {
    pub fn new(semaphore: usize) -> Self {
        Self {
            semaphore,
            buffered_logs: vec![],
        }
    }
}

impl Logger for BeaconLogger {
    fn log(&mut self, log: LogSignal) {
        self.buffered_logs.push(log);
        vm().proxy().signal_semaphore(self.semaphore);
    }

    fn poll_all(&mut self) -> Vec<LogSignal> {
        mem::replace(&mut self.buffered_logs, vec![])
    }

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStartBeacon() {
    let vm = vm();
    let proxy = vm.proxy();

    let semaphore = proxy.stack_integer_value(StackOffset::new(0)) as usize;

    let mut logger = VM_LOGGER.lock().unwrap();
    logger.set_logger(Box::new(BeaconLogger::new(semaphore)));

    proxy.method_return_boolean(true);
}
