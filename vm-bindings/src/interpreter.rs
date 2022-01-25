use crate::bindings::{
    free, installErrorHandlers, logLevel, osCogStackPageHeadroom, pluginExports,
    registerCurrentThreadToHandleExceptions, setProcessArguments, setProcessEnvironmentVector,
    sqExport, sqInt, vm_init, vm_main_with_parameters, vm_run_interpreter,
};
use crate::prelude::{Handle, NativeAccess, NativeDrop, NativeTransmutable};
use crate::InterpreterParameters;
use anyhow::{bail, Result};
use std::ffi::{c_void, CStr, CString};
use std::fmt::{Debug, Display, Formatter};
use std::os::raw::c_int;
use std::panic;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct PharoInterpreter {
    parameters: InterpreterParameters,
}

unsafe impl Send for PharoInterpreter {}
unsafe impl Sync for PharoInterpreter {}

impl PharoInterpreter {
    pub fn new(parameters: InterpreterParameters) -> Self {
        Self { parameters }
    }

    /// Set the logLevel to use in the VM
    pub fn log_level(&self, level: LogLevel) {
        unsafe { logLevel(level as u8 as c_int) };
    }

    /// Launch the vm in the main process
    pub fn start(&self) -> Result<()> {
        self.prepare_environment();
        self.run();
        Ok(())
    }

    /// Launch the vm in a worker thread returning the Join handle
    pub fn start_in_worker(self: Arc<Self>) -> Result<JoinHandle<Result<()>>> {
        self.prepare_environment();

        let vm = self.clone();
        std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(move || vm.run())
            .map_err(|error| error.into())
    }

    fn prepare_environment(&self) {
        unsafe { setProcessEnvironmentVector(self.parameters.native().environmentVector) };
        unsafe {
            setProcessArguments(
                self.parameters.native().processArgc,
                self.parameters.native().processArgv,
            )
        };
        unsafe { osCogStackPageHeadroom() };
    }

    /// Initializes the vm and runs the interpreter.
    /// Can be executed from any thread
    fn run(&self) -> Result<()> {
        self.init()?;
        self.register_current_thread_to_handle_exceptions();
        self.run_interpreter();
        Ok(())
    }

    /// Initializes the vm with the current parameters
    fn init(&self) -> Result<()> {
        if !unsafe { vm_init(self.parameters.native_mut_force()) != 0 } {
            return bail!(
                "Error opening image file: {}",
                self.parameters.image_file_name()
            );
        }
        Ok(())
    }

    fn register_current_thread_to_handle_exceptions(&self) {
        unsafe { registerCurrentThreadToHandleExceptions() };
    }

    /// Run the interpreter until it exits. Blocks the current process
    fn run_interpreter(&self) {
        unsafe {
            vm_run_interpreter();
        };
    }

    pub fn vm_exports(&self) -> &[Export] {
        let plugin_exports: [*mut sqExport; 3usize] = unsafe { pluginExports };

        let vm_exports_ptr: *const Export = plugin_exports[0] as *const Export;
        let length = detect_length(plugin_exports[0]);
        unsafe { std::slice::from_raw_parts(vm_exports_ptr, length) }
    }
}

fn detect_length(exports: *const sqExport) -> usize {
    let exports = exports as *const sqExport;

    let mut length = 0 as usize;
    loop {
        let each_export_ptr = unsafe { exports.offset(length as isize) };
        if each_export_ptr == std::ptr::null_mut() {
            break;
        }
        let each_export = unsafe { &*each_export_ptr };
        if !each_export.is_valid() {
            println!("export is not valid! {:?}", each_export);
            break;
        }
        length = length + 1;
    }
    length
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectPointer(sqInt);
impl NativeTransmutable<sqInt> for ObjectPointer {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectFieldIndex(sqInt);
impl NativeTransmutable<sqInt> for ObjectFieldIndex {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct StackOffset(sqInt);
impl NativeTransmutable<sqInt> for StackOffset {}

pub type Export = Handle<sqExport>;
impl NativeDrop for sqExport {
    fn drop(&mut self) {
        self.take_plugin_name();
        self.take_primitive_name();
    }
}

impl Export {
    pub fn plugin_name(&self) -> &str {
        let plugin_name_ptr: *mut std::os::raw::c_char = self.native().pluginName;
        unsafe { CStr::from_ptr(plugin_name_ptr) }.to_str().unwrap()
    }

    pub fn primitive_name(&self) -> &str {
        let primitive_name_ptr: *mut std::os::raw::c_char = self.native().primitiveName;
        unsafe { CStr::from_ptr(primitive_name_ptr) }
            .to_str()
            .unwrap()
    }

    pub fn primitive_address(&self) -> *const std::os::raw::c_void {
        self.native().primitiveAddress
    }
}

impl sqExport {
    fn take_plugin_name(&mut self) -> String {
        let plugin_name_ptr: *mut std::os::raw::c_char = self.pluginName;
        self.pluginName = std::ptr::null_mut();
        unsafe { CString::from_raw(plugin_name_ptr) }
            .into_string()
            .unwrap()
    }

    fn take_primitive_name(&mut self) -> String {
        let primitive_name_ptr: *mut std::os::raw::c_char = self.primitiveName;
        self.primitiveName = std::ptr::null_mut();
        unsafe { CString::from_raw(primitive_name_ptr) }
            .into_string()
            .unwrap()
    }

    fn is_valid(&self) -> bool {
        if self.primitiveName == std::ptr::null_mut() {
            return false;
        }

        if self.pluginName == std::ptr::null_mut() {
            return false;
        }

        if self.primitiveAddress == std::ptr::null_mut() {
            return false;
        }

        true
    }
}

impl Debug for Export {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Export")
            .field("plugin_name", &self.plugin_name())
            .field("primitive_name", &self.primitive_name())
            .field("primitive_address", &self.primitive_address())
            .finish()
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}
