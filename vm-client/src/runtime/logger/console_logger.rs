use crate::{vm, LogSignal, Logger, NullLogger, VM_LOGGER};
use num_traits::FromPrimitive;
use std::any::Any;
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::mem::size_of;
pub use std::os::raw::{c_char, c_int};
use vm_bindings::{LogLevel, ObjectFieldIndex, StackOffset};

use chrono::Local;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ConsoleLogger;

impl ConsoleLogger {
    pub fn new() -> Self {
        Self
    }
}

impl Logger for ConsoleLogger {
    #[cfg(feature = "colored_terminal")]
    fn log(&mut self, log: LogSignal) {
        use colored::*;
        println!(
            "{} {} {} - {}",
            Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .bright_black(),
            log.log_type.bright_black().bold(),
            format!("{}:{}", log.file_name, log.line).bright_black(),
            log.message.trim()
        );
    }
    #[cfg(not(feature = "colored_terminal"))]
    fn log(&mut self, log: LogSignal) {
        println!(
            "{} {} {} - {}",
            Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            log.log_type,
            format!("{}:{}", log.file_name, log.line),
            log.message.trim()
        );
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
pub fn primitiveStartConsoleLogger() {
    let vm = vm();
    let proxy = vm.proxy();

    let mut logger = VM_LOGGER.lock().unwrap();
    logger.set_logger(Box::new(ConsoleLogger::new()));

    proxy.method_return_boolean(true);
}
