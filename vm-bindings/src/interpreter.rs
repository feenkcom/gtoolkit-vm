#![feature(catch_panic)]

use crate::bindings::{
    installErrorHandlers, logLevel, osCogStackPageHeadroom,
    registerCurrentThreadToHandleExceptions, setProcessArguments, setProcessEnvironmentVector,
    vm_init, vm_main_with_parameters, vm_run_interpreter,
};
use crate::prelude::NativeAccess;
use crate::InterpreterParameters;
use anyhow::{bail, Result};
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
        self.log_level(LogLevel::Trace);
        self.prepare_environment();
        self.run();
        Ok(())
    }

    /// Launch the vm in a worker thread returning the Join handle
    pub fn start_in_worker(self: Arc<Self>) -> Result<JoinHandle<Result<()>>> {
        self.log_level(LogLevel::Trace);
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
