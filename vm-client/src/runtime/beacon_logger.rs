use crate::vm;
use num_traits::FromPrimitive;
use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::mem::size_of;
pub use std::os::raw::{c_char, c_int};
use vm_bindings::{LogLevel, ObjectFieldIndex, StackOffset};

use std::sync::Mutex;

lazy_static! {
    static ref BEACON_LOGGER: Mutex<Option<BeaconLogger>> = Mutex::new(None);
}

#[derive(Debug)]
struct BeaconLogger {
    semaphore: usize,
    logs: Vec<BeaconLog>,
}

impl BeaconLogger {
    pub fn new(semaphore: usize) -> Self {
        Self {
            semaphore,
            logs: vec![],
        }
    }

    pub fn log(&mut self, log: BeaconLog) {
        self.logs.push(log);
        vm().proxy().signal_semaphore(self.semaphore);
    }

    pub fn poll_all(&mut self) -> Vec<BeaconLog> {
        mem::replace(&mut self.logs, vec![])
    }
}

#[derive(Debug)]
struct BeaconLog {
    level: LogLevel,
    file_name: String,
    function_name: String,
    line: usize,
    message: String,
}

#[no_mangle]
pub unsafe extern "C" fn beacon_logger(
    level: c_int,
    file_name: *const c_char,
    function_name: *const c_char,
    line: c_int,
    message: *const c_char,
) {
    let mut logger = BEACON_LOGGER.lock().unwrap();

    if let Some(ref mut logger) = *logger {
        let log_level = LogLevel::from_u8(level as u8).unwrap();
        let file_name = CStr::from_ptr(file_name).to_string_lossy().to_string();
        let function_name = CStr::from_ptr(function_name).to_string_lossy().to_string();
        let message = CStr::from_ptr(message).to_string_lossy().to_string();

        logger.log(BeaconLog {
            level: log_level,
            file_name,
            function_name,
            line: line as usize,
            message,
        })
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveSetBeaconLogger() {
    let vm = vm();
    let proxy = vm.proxy();

    vm.interpreter().set_logger(Some(beacon_logger));

    let semaphore = proxy.stack_integer_value(StackOffset::new(0)) as usize;

    let mut locked_beacon = BEACON_LOGGER.lock().unwrap();
    *locked_beacon = Some(BeaconLogger::new(semaphore));

    proxy.method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveRemoveBeaconLogger() {
    let vm = vm();
    let proxy = vm.proxy();

    vm.interpreter().set_logger(None);

    let mut locked_beacon = BEACON_LOGGER.lock().unwrap();
    *locked_beacon = None;

    proxy.method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePollBeaconLogger() {
    let proxy = vm().proxy();

    let logs = BEACON_LOGGER
        .lock()
        .unwrap()
        .as_mut()
        .map_or_else(|| vec![], |logger| logger.poll_all());

    let return_array = proxy.instantiate_indexable_class_of_size(proxy.class_array(), logs.len());
    for (index, log) in logs.iter().enumerate() {
        let each_log_array = proxy.instantiate_indexable_class_of_size(proxy.class_array(), 3);

        let file_name = proxy.new_string(log.file_name.as_str());
        let function_name = proxy.new_string(log.function_name.as_str());
        let message = proxy.new_string(log.message.as_str());

        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(1), file_name);
        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(2), function_name);
        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(3), message);

        proxy.item_at_put(
            return_array,
            ObjectFieldIndex::new(index + 1),
            each_log_array,
        );
    }
    proxy.method_return_value(return_array);
}
