use crate::vm;
use std::any::Any;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;
use vm_bindings::{ObjectFieldIndex, StackOffset};

lazy_static! {
    pub static ref VM_LOGGER: Mutex<VirtualMachineLogger> = Mutex::new(VirtualMachineLogger::new());
}

#[derive(Debug)]
pub struct VirtualMachineLogger {
    enabled_log_types: HashSet<CString>,
    logger: Box<dyn Logger>,
}

impl VirtualMachineLogger {
    pub fn new() -> Self {
        Self {
            enabled_log_types: Default::default(),
            logger: Box::new(NullLogger),
        }
    }

    pub fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }

    pub fn enable_type(&mut self, log_type: CString) {
        self.enabled_log_types.insert(log_type);
    }

    pub fn enabled_types(&self) -> Vec<String> {
        self.enabled_log_types
            .iter()
            .map(|each| each.to_string_lossy().to_string())
            .collect()
    }

    pub fn should_log(&self, log_type: &CStr) -> bool {
        if self.logger.is_null() {
            return false;
        }
        self.enabled_log_types.contains(log_type)
    }

    pub fn log(&mut self, log: LogSignal) {
        self.logger.log(log);
    }

    pub fn poll_all(&mut self) -> Vec<LogSignal> {
        self.logger.poll_all()
    }

    pub fn logger<T: 'static>(&self) -> Option<&T> {
        self.logger.any().downcast_ref()
    }

    pub fn logger_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.logger.any_mut().downcast_mut()
    }
}

pub trait Logger: Debug + Send {
    fn is_null(&self) -> bool {
        false
    }
    fn log(&mut self, log: LogSignal);
    fn poll_all(&mut self) -> Vec<LogSignal> {
        vec![]
    }
    fn any(&self) -> &dyn Any;
    fn any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct LogSignal {
    pub log_type: String,
    pub file_name: String,
    pub function_name: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug)]
pub struct NullLogger;
impl Logger for NullLogger {
    fn is_null(&self) -> bool {
        true
    }

    fn log(&mut self, log: LogSignal) {}

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[no_mangle]
pub unsafe extern "C" fn log_signal(
    log_type: *const c_char,
    file_name: *const c_char,
    function_name: *const c_char,
    line: c_int,
    message: *const c_char,
) {
    let mut logger = VM_LOGGER.lock().unwrap();

    let log_type = CStr::from_ptr(log_type).to_string_lossy().to_string();
    let file_name = CStr::from_ptr(file_name).to_string_lossy().to_string();
    let function_name = CStr::from_ptr(function_name).to_string_lossy().to_string();
    let message = CStr::from_ptr(message).to_string_lossy().to_string();

    logger.log(LogSignal {
        log_type,
        file_name,
        function_name,
        line: line as usize,
        message,
    });
}

#[no_mangle]
pub unsafe extern "C" fn should_log_signal(log_type: *const c_char) -> bool {
    let mut logger = VM_LOGGER.lock().unwrap();
    logger.should_log(CStr::from_ptr(log_type))
}

#[no_mangle]
pub unsafe extern "C" fn should_log_all_signals(log_type: *const c_char) -> bool {
    true
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveEnableLogSignal() {
    let proxy = vm().proxy();
    let mut logger = VM_LOGGER.lock().unwrap();

    let smalltalk_string = proxy.stack_object_value(StackOffset::new(0));

    match proxy.cstring_value_of(smalltalk_string) {
        None => {
            proxy.primitive_fail();
        }
        Some(cstring) => {
            logger.enable_type(cstring);
            proxy.method_return_boolean(true);
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetEnabledLogSignals() {
    let proxy = vm().proxy();

    let logs = VM_LOGGER.lock().unwrap().enabled_types();

    let return_array = proxy.instantiate_indexable_class_of_size(proxy.class_array(), logs.len());
    for (index, log_type) in logs.iter().enumerate() {
        let each_type = proxy.new_string(log_type);
        proxy.item_at_put(return_array, ObjectFieldIndex::new(index + 1), each_type);
    }
    proxy.method_return_value(return_array);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePollLogger() {
    let proxy = vm().proxy();

    let logs = VM_LOGGER.lock().unwrap().poll_all();

    let return_array = proxy.instantiate_indexable_class_of_size(proxy.class_array(), logs.len());
    for (index, log) in logs.iter().enumerate() {
        let each_log_array = proxy.instantiate_indexable_class_of_size(proxy.class_array(), 4);

        let log_type = proxy.new_string(log.log_type.as_str());
        let file_name = proxy.new_string(log.file_name.as_str());
        let function_name = proxy.new_string(log.function_name.as_str());
        let message = proxy.new_string(log.message.as_str());

        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(1), log_type);
        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(2), file_name);
        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(3), function_name);
        proxy.item_at_put(each_log_array, ObjectFieldIndex::new(4), message);

        proxy.item_at_put(
            return_array,
            ObjectFieldIndex::new(index + 1),
            each_log_array,
        );
    }
    proxy.method_return_value(return_array);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStopLogger() {
    let vm = vm();
    let proxy = vm.proxy();

    let mut logger = VM_LOGGER.lock().unwrap();
    logger.set_logger(Box::new(NullLogger));

    proxy.method_return_boolean(true);
}
