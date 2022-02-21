use crate::bindings::{
    free, getOsCogStackPageHeadroom as osCogStackPageHeadroom,
    getSqGetInterpreterProxy as sqGetInterpreterProxy, getVMExports, installErrorHandlers,
    logLevel, registerCurrentThreadToHandleExceptions, setProcessArguments,
    setProcessEnvironmentVector, setVMExports, sqExport, sqInt, vm_init,
    vm_parameters_ensure_interactive_image_parameter, vm_run_interpreter, VirtualMachine,
    setVmRunOnWorkerThread
};
use crate::prelude::NativeAccess;
use crate::{InterpreterParameters, InterpreterProxy, NamedPrimitive};
use anyhow::{bail, Result};
use std::fmt::Debug;
use std::os::raw::c_int;
use std::panic;
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
        let interpreter = Self { parameters };
        interpreter.initialize_vm_exports();
        interpreter
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
        self.mark_as_running_in_worker_thread();

        let vm = self.clone();
        std::thread::Builder::new()
            .name("PharoVM".to_string())
            .stack_size(512 * 1024 * 1024)
            .spawn(move || vm.run())
            .map_err(|error| error.into())
    }

    fn prepare_environment(&self) {
        unsafe {
            vm_parameters_ensure_interactive_image_parameter(self.parameters.native_mut_force())
        };
        unsafe { installErrorHandlers() };
        unsafe {
            setProcessArguments(
                self.parameters.native().processArgc,
                self.parameters.native().processArgv,
            )
        };
        unsafe { setProcessEnvironmentVector(self.parameters.native().environmentVector) };
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

    /// Mark the interpreter as running in a worker thread
    fn mark_as_running_in_worker_thread(&self) {
        unsafe {
            setVmRunOnWorkerThread(1);
        }
    }

    pub fn proxy(&self) -> &InterpreterProxy {
        unsafe { InterpreterProxy::from_native_ref(&*sqGetInterpreterProxy()) }
    }

    /// Return a slice of all named primitives that are exported from the vm
    pub fn vm_exports(&self) -> &[NamedPrimitive] {
        let vm_exports_ptr: *const NamedPrimitive =
            unsafe { getVMExports() } as *const NamedPrimitive;
        let length = NamedPrimitive::detect_exports_length(vm_exports_ptr);
        unsafe { std::slice::from_raw_parts(vm_exports_ptr, length) }
    }

    pub fn add_vm_export(&self, export: NamedPrimitive) {
        let vm_exports_ptr: *mut NamedPrimitive = unsafe { getVMExports() } as *mut NamedPrimitive;
        let length = NamedPrimitive::detect_exports_length(vm_exports_ptr);

        let mut new_vm_exports = self
            .vm_exports()
            .iter()
            .map(|each| each.clone())
            .collect::<Vec<NamedPrimitive>>();
        new_vm_exports.push(export);
        new_vm_exports.push(NamedPrimitive::null());

        // the `length + 1` element is part if the vector, we should take it into account
        let mut vm_exports = unsafe { Vec::from_raw_parts(vm_exports_ptr, length + 1, length + 1) };
        drop(vm_exports);

        new_vm_exports.shrink_to_fit();
        if new_vm_exports.len() != new_vm_exports.capacity() {
            panic!("Failed to shrink the vector");
        }
        let vm_exports_ptr = new_vm_exports.as_mut_ptr() as *mut sqExport;
        std::mem::forget(new_vm_exports);
        unsafe { setVMExports(vm_exports_ptr) };
    }

    /// re-allocate the vm-exports memory using rust allocator so that we can modify the exports
    fn initialize_vm_exports(&self) {
        let vm_exports_ptr: *const NamedPrimitive =
            unsafe { getVMExports() } as *const NamedPrimitive;
        let length = NamedPrimitive::detect_exports_length(vm_exports_ptr);
        let vm_exports = unsafe { std::slice::from_raw_parts(vm_exports_ptr, length + 1) };
        let mut vm_exports_vec = vm_exports
            .iter()
            .map(|each_export| each_export.clone())
            .collect::<Vec<NamedPrimitive>>();
        vm_exports_vec.shrink_to_fit();
        if vm_exports_vec.len() != vm_exports_vec.capacity() {
            panic!("Failed to shrink the vector");
        }
        let vm_exports_ptr = vm_exports_vec.as_mut_ptr() as *mut sqExport;
        std::mem::forget(vm_exports_vec);
        unsafe { setVMExports(vm_exports_ptr) };
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
