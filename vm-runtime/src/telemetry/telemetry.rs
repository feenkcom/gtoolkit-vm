use crate::{vm, ProcessSwitchTelemetry};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Instant;
use vm_bindings::bindings::{sqInt, InterpreterTelemetry};
use vm_bindings::{Smalltalk, StackOffset};
use vm_object_model::{AnyObject, RawObjectPointer};

static TELEMETRY_INSTANCE: OnceCell<Mutex<GlobalTelemetry>> = OnceCell::new();

struct GlobalTelemetry {
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

    pub fn add_telemetry(&mut self, mut telemetry: Box<dyn AbstractTelemetry>) -> usize {
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
        old_process: AnyObject,
        new_process: AnyObject,
    ) {
        self.receive_signal(TelemetrySignal::ContextSwitch(ContextSwitchSignal {
            timestamp: Instant::now(),
            old_process: old_process.raw_header(),
            new_process: new_process.raw_header(),
        }));
    }

    pub fn receive_semaphore_wait_signal(
        &mut self,
        semaphore: AnyObject,
        process: AnyObject,
        is_locked: bool,
    ) {
        self.receive_signal(TelemetrySignal::SemaphoreWait(SemaphoreWaitSignal {
            timestamp: Instant::now(),
            semaphore: semaphore.raw_header(),
            process: process.raw_header(),
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
    pub old_process: RawObjectPointer,
    pub new_process: RawObjectPointer,
}

#[derive(Debug, Clone)]
pub struct SemaphoreWaitSignal {
    pub timestamp: Instant,
    pub semaphore: RawObjectPointer,
    pub process: RawObjectPointer,
    pub is_locked: bool,
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStartProcessSwitchTelemetry() {
    let telemetry_pointer = Smalltalk::stack_value(StackOffset::new(0));
    let process_switch_telemetry =
        ProcessSwitchTelemetry::new(RawObjectPointer::new(telemetry_pointer.as_i64()));

    TELEMETRY_INSTANCE
        .get_or_init(|| {
            let telemetry = GlobalTelemetry::init();
            Mutex::new(telemetry)
        })
        .lock()
        .add_telemetry(Box::new(process_switch_telemetry));

    Smalltalk::method_return_boolean(true);
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
        telemetry.lock().receive_context_switch_signal(
            AnyObject::from(old_process),
            AnyObject::from(new_process),
        );
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
        telemetry.lock().receive_semaphore_wait_signal(
            AnyObject::from(semaphore),
            AnyObject::from(process),
            is_locked != 0,
        );
    }
}
