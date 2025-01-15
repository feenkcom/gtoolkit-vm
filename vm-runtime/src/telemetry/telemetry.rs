use crate::vm;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Instant;
use vm_bindings::bindings::{sqInt, InterpreterTelemetry};
use vm_bindings::{Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, ObjectRef, RawObjectPointer};

static TELEMETRY_INSTANCE: OnceCell<Mutex<GlobalTelemetry>> = OnceCell::new();

pub struct GlobalTelemetry {
    telemetries: HashMap<usize, Box<dyn AbstractTelemetry>>,
    next_instance_id: usize,
}

impl GlobalTelemetry {
    pub fn init() -> Self {
        Self {
            telemetries: HashMap::new(),
            next_instance_id: 1,
        }
    }

    pub fn register(telemetry: impl AbstractTelemetry + 'static) {
        TELEMETRY_INSTANCE
            .get_or_init(|| {
                let telemetry = Self::init();
                Mutex::new(telemetry)
            })
            .lock()
            .add_telemetry(Box::new(telemetry));
    }

    fn add_telemetry(&mut self, mut telemetry: Box<dyn AbstractTelemetry>) -> usize {
        if self.telemetries.is_empty() {
            let interpreter = vm().interpreter();
            interpreter.set_telemetry(self.as_interpreter_telemetry());
            interpreter.enable_telemetry();
        }

        let id = self.next_instance_id;
        telemetry.assign_id(id);
        self.telemetries.insert(id, telemetry);
        self.next_instance_id += 1;
        id
    }

    pub fn remove_telemetry(&mut self, id: usize) {
        self.telemetries.remove(&id);
        if self.telemetries.is_empty() {
            let interpreter = vm().interpreter();
            interpreter.disable_telemetry();
            interpreter.take_telemetry();
        }
    }

    pub fn receive_context_switch_signal(
        &mut self,
        old_process: ObjectRef,
        new_process: ObjectRef,
    ) {
        self.receive_signal(TelemetrySignal::ContextSwitch(ContextSwitchSignal {
            timestamp: Instant::now(),
            old_process,
            new_process,
        }));
    }

    pub fn receive_semaphore_wait_signal(
        &mut self,
        semaphore: ObjectRef,
        process: ObjectRef,
        is_locked: bool,
    ) {
        self.receive_signal(TelemetrySignal::SemaphoreWait(SemaphoreWaitSignal {
            timestamp: Instant::now(),
            semaphore,
            process,
            is_locked,
        }));
    }

    pub fn receive_signal(&mut self, signal: TelemetrySignal) {
        self.telemetries
            .values_mut()
            .for_each(|telemetry| telemetry.receive_signal(&signal));
    }

    pub fn as_interpreter_telemetry(&self) -> InterpreterTelemetry {
        InterpreterTelemetry {
            payload: std::ptr::null_mut(),
            sendFn: None,
            returnFn: None,
            primitiveActivationFn: None,
            activateMachineMethodFn: None,
            beginMachineMethodFn: None,
            contextSwitchFn: Some(telemetry_receive_context_switch_signal),
            debugRecordClassFn: None,
            debugRecordSelectorFn: None,
            semaphoreWaitFn: Some(telemetry_receive_semaphore_wait_signal),
        }
    }
}

pub trait AbstractTelemetry: Send + Sync {
    fn receive_signal(&mut self, signal: &TelemetrySignal);
    fn assign_id(&mut self, id: usize);
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TelemetrySignal {
    ContextSwitch(ContextSwitchSignal),
    SemaphoreWait(SemaphoreWaitSignal),
}

#[derive(Debug, Clone)]
pub struct ContextSwitchSignal {
    pub timestamp: Instant,
    pub old_process: ObjectRef,
    pub new_process: ObjectRef,
}

#[derive(Debug, Clone)]
pub struct SemaphoreWaitSignal {
    pub timestamp: Instant,
    pub semaphore: ObjectRef,
    pub process: ObjectRef,
    pub is_locked: bool,
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStopTelemetry() {
    let telemetry_id = Smalltalk::stack_integer_value(StackOffset::new(0)) as usize;
    if let Some(telemetry) = TELEMETRY_INSTANCE.get() {
        telemetry.lock().remove_telemetry(telemetry_id);
    }

    Smalltalk::method_return_value(Smalltalk::true_object());
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_context_switch_signal(
    _nothing: *mut c_void,
    old_process: sqInt,
    new_process: sqInt,
) {
    if let Some(telemetry) = TELEMETRY_INSTANCE.get() {
        let old_process_ref = AnyObjectRef::from(RawObjectPointer::new(old_process));
        let new_process_ref = AnyObjectRef::from(RawObjectPointer::new(new_process));

        let old_process = match old_process_ref.as_object() {
            Ok(old_process) => old_process,
            Err(error) => {
                error!("Failed to get old_process object: {:?}", error);
                return;
            }
        };

        let new_process = match new_process_ref.as_object() {
            Ok(new_process) => new_process,
            Err(error) => {
                error!("Failed to get new_process object: {:?}", error);
                return;
            }
        };

        telemetry
            .lock()
            .receive_context_switch_signal(old_process, new_process);
    }
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_semaphore_wait_signal(
    _nothing: *mut c_void,
    semaphore: sqInt,
    process: sqInt,
    is_locked: u8,
) {
    if let Some(telemetry) = TELEMETRY_INSTANCE.get() {
        let semaphore_ref = AnyObjectRef::from(RawObjectPointer::new(semaphore));
        let process_ref = AnyObjectRef::from(RawObjectPointer::new(process));

        let semaphore = match semaphore_ref.as_object() {
            Ok(semaphore) => semaphore,
            Err(error) => {
                error!("Failed to get semaphore object: {:?}", error);
                return;
            }
        };

        let process = match process_ref.as_object() {
            Ok(process) => process,
            Err(error) => {
                error!("Failed to get process object: {:?}", error);
                return;
            }
        };

        telemetry
            .lock()
            .receive_semaphore_wait_signal(semaphore, process, is_locked != 0);
    }
}
