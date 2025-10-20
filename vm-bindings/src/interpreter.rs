use crate::bindings::{
    disableTelemetry, enableTelemetry, exportOsCogStackPageHeadroom as osCogStackPageHeadroom,
    exportSqGetInterpreterProxy as sqGetInterpreterProxy, exportStatFullGCUsecs as statFullGCUsecs,
    exportStatScavengeGCUsecs as statScavengeGCUsecs, getVMExports, installErrorHandlers,
    registerCurrentThreadToHandleExceptions, setLogger, setProcessArguments,
    setProcessEnvironmentVector, setShouldLog, setTelemetry, setVMExports, setVmRunOnWorkerThread,
    takeTelemetry, vm_init, vm_parameters_ensure_interactive_image_parameter, vm_run_interpreter,
    InterpreterTelemetry,
};
use crate::parameters::InterpreterParameters;
use crate::prelude::NativeAccess;
use crate::{InterpreterConfiguration, InterpreterProxy, NamedPrimitive};
use anyhow::{bail, Result};
use std::fmt::Debug;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{panic, slice};

#[derive(Debug)]
pub struct PharoInterpreter {
    configuration: InterpreterConfiguration,
}

unsafe impl Send for PharoInterpreter {}
unsafe impl Sync for PharoInterpreter {}

impl PharoInterpreter {
    pub fn new(configuration: InterpreterConfiguration) -> Self {
        let interpreter = Self { configuration };
        interpreter.initialize_vm_exports();
        interpreter
    }

    /// Set the logger to be used
    pub fn set_logger(
        &self,
        logger: Option<
            unsafe extern "C" fn(
                log_type: *const c_char,
                file_name: *const c_char,
                function_name: *const c_char,
                line: c_int,
                message: *const c_char,
            ),
        >,
    ) {
        unsafe { setLogger(logger) };
    }

    /// Set the function that should be called to determine if a given log type should be logged
    pub fn set_should_log(
        &self,
        should_log: Option<unsafe extern "C" fn(log_type: *const c_char) -> bool>,
    ) {
        unsafe { setShouldLog(should_log) };
    }

    pub fn set_telemetry(&self, interpreter_telemetry: InterpreterTelemetry) {
        unsafe { setTelemetry(Box::into_raw(Box::new(interpreter_telemetry))) };
    }

    pub fn take_telemetry(&self) -> Option<InterpreterTelemetry> {
        let telemetry = unsafe { takeTelemetry() };
        if telemetry.is_null() {
            None
        } else {
            Some(unsafe { *Box::from_raw(telemetry) })
        }
    }

    pub fn enable_telemetry(&self) {
        unsafe { enableTelemetry() };
    }

    pub fn disable_telemetry(&self) {
        unsafe { disableTelemetry() };
    }

    /// Launch the vm according to the configuration
    pub fn start(self: Arc<Self>) -> Result<Option<JoinHandle<Result<()>>>> {
        let parameters = self.configuration.create_interpreter_parameters();
        if self.configuration.is_worker_thread() {
            Ok(Some(self.start_in_worker_thread(parameters)?))
        } else {
            self.start_in_main_thread(parameters)?;
            Ok(None)
        }
    }

    /// Launch the vm in the main process
    fn start_in_main_thread(self: Arc<Self>, parameters: InterpreterParameters) -> Result<()> {
        self.prepare_environment(&parameters);
        self.run(parameters)?;
        Ok(())
    }

    /// Launch the vm in a worker thread returning the Join handle
    fn start_in_worker_thread(
        self: Arc<Self>,
        parameters: InterpreterParameters,
    ) -> Result<JoinHandle<Result<()>>> {
        self.prepare_environment(&parameters);
        self.mark_as_running_in_worker_thread();

        let vm = self.clone();
        std::thread::Builder::new()
            .name("PharoVM".to_string())
            .stack_size(self.guess_stack_size())
            .spawn(move || vm.run(parameters))
            .map_err(|error| error.into())
    }

    fn guess_stack_size(&self) -> usize {
        32 * 1024 * 1024
    }

    fn prepare_environment(&self, parameters: &InterpreterParameters) {
        unsafe { vm_parameters_ensure_interactive_image_parameter(parameters.native_mut_force()) };
        if self.configuration.should_print_stack_on_signals() {
            unsafe { installErrorHandlers() };
        }
        unsafe {
            setProcessArguments(
                parameters.native().processArgc,
                parameters.native().processArgv,
            )
        };
        unsafe { setProcessEnvironmentVector(parameters.native().environmentVector) };
        unsafe { osCogStackPageHeadroom() };
    }

    /// Initializes the vm and runs the interpreter.
    /// Can be executed from any thread
    fn run(&self, parameters: InterpreterParameters) -> Result<()> {
        self.init(parameters)?;
        self.register_current_thread_to_handle_exceptions();
        self.run_interpreter();
        Ok(())
    }

    /// Initializes the vm with the current parameters
    fn init(&self, parameters: InterpreterParameters) -> Result<()> {
        if !unsafe { vm_init(parameters.native_mut_force()) != 0 } {
            return bail!(
                "Error opening image file: {}",
                self.configuration.image().display()
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
        unsafe { slice::from_raw_parts(vm_exports_ptr, length) }
    }

    pub fn add_vm_export(&self, export: NamedPrimitive) {
        let vm_exports_ptr: *mut NamedPrimitive = unsafe { getVMExports() } as *mut NamedPrimitive;
        let length = NamedPrimitive::detect_exports_length(vm_exports_ptr);
        let new_byte_size = (length + 1) * size_of::<NamedPrimitive>();

        let new_ptr =
            unsafe { libc::realloc(vm_exports_ptr as _, new_byte_size) } as *mut NamedPrimitive;
        if new_ptr.is_null() {
            panic!("Failed to reallocate memory for named primitives");
        }

        let mut vm_exports = unsafe { slice::from_raw_parts_mut(new_ptr, length + 1) };
        vm_exports[length - 1] = export;
        vm_exports[length] = NamedPrimitive::null();

        unsafe { setVMExports(new_ptr as _) };
    }

    /// Return the total amount of microseconds spent on full garbage collection
    pub fn full_gc_microseconds(&self) -> u64 {
        unsafe { statFullGCUsecs().into() }
    }

    /// Return the total amount of microseconds spent on scavenge garbage collection
    pub fn scavenge_gc_microseconds(&self) -> u64 {
        unsafe { statScavengeGCUsecs().into() }
    }

    /// re-allocate the vm-exports memory using rust allocator so that we can modify the exports
    fn initialize_vm_exports(&self) {
        let vm_exports_ptr: *const NamedPrimitive =
            unsafe { getVMExports() } as *const NamedPrimitive;
        let length = NamedPrimitive::detect_exports_length(vm_exports_ptr) + 1;

        let byte_size = length * size_of::<NamedPrimitive>();

        let mut new_vm_exports_ptr = unsafe { libc::malloc(byte_size) } as *mut NamedPrimitive;
        if new_vm_exports_ptr.is_null() {
            panic!("Failed to allocate memory for named primitives");
        }

        let new_vm_exports = unsafe { slice::from_raw_parts_mut(new_vm_exports_ptr, length + 1) };
        for i in 0..length {
            let previous_primitive = unsafe { &*vm_exports_ptr.offset(i as isize) };
            new_vm_exports[i].native_mut().pluginName = previous_primitive.native().pluginName;
            new_vm_exports[i].native_mut().primitiveName =
                previous_primitive.native().primitiveName;
            new_vm_exports[i].native_mut().primitiveAddress =
                previous_primitive.native().primitiveAddress;
        }

        // there is no need to free previous vm exports because it wasn't allocated

        unsafe { setVMExports(new_vm_exports_ptr as _) };
    }
}

#[derive(Debug, Clone, FromPrimitive, ToPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}
