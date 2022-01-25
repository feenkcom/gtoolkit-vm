use crate::EventLoopMessage;
use anyhow::Result;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use vm_bindings::{InterpreterParameters, PharoInterpreter};

#[derive(Debug)]
pub struct VirtualMachine {
    interpreter: Arc<PharoInterpreter>,
    event_loop_sender: Sender<EventLoopMessage>,
}

impl VirtualMachine {
    pub fn new(
        parameters: InterpreterParameters,
        event_loop_sender: Sender<EventLoopMessage>,
    ) -> Self {
        Self {
            interpreter: Arc::new(PharoInterpreter::new(parameters)),
            event_loop_sender,
        }
    }

    /// Starts the interpreter in a worker thread
    pub fn start(&self) -> Result<JoinHandle<Result<()>>> {
        self.interpreter.clone().start_in_worker()
    }
}
