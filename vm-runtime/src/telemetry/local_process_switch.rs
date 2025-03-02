use crate::objects::OrderedCollectionRef;
use crate::{AbstractTelemetry, ApplicationError, ContextSwitchSignal, GlobalTelemetry, ComputationSignal, Result, SemaphoreWaitSignal, TelemetrySignal, ContextSignal};
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[repr(C)]
pub struct LocalProcessSwitchTelemetry {
    this: Object,
    id: Immediate,
    signals: OrderedCollectionRef,
    current_process: ObjectRef,
    context_switch_signal_class: ObjectRef,
    semaphore_wait_signal_class: ObjectRef,
}

impl LocalProcessSwitchTelemetry {
    fn receive_context_switch_signal(&mut self, signal: &ContextSwitchSignal) {
        if signal.old_process == self.current_process {
            // switches away
            self.add_context_switch_signal(false);
        } else if signal.new_process == self.current_process {
            // switches back
            self.add_context_switch_signal(true);
        }
    }

    fn receive_computation_signal(&mut self, signal: &ComputationSignal) {
        if signal.process == self.current_process {
            //todo self.add_semaphore_wait_signal(signal);
        }
    }

    fn receive_context_signal(&mut self, signal: &ContextSignal) {
        if signal.process == self.current_process {
            //todo self.add_semaphore_wait_signal(signal);
        }
    }

    fn receive_semaphore_wait_signal(&mut self, signal: &SemaphoreWaitSignal) {
        if signal.is_locked && signal.process == self.current_process {
            self.add_semaphore_wait_signal(signal);
        }
    }

    fn add_context_switch_signal(&mut self, alive: bool) {
        self.add_signal(self.context_switch_signal_class, |signal_object| {
            signal_object.inst_var_at_put(
                2,
                RawObjectPointer::new(Smalltalk::primitive_bool_object(alive).as_i64()),
            );
        });
    }

    fn add_semaphore_wait_signal(&mut self, signal: &SemaphoreWaitSignal) {
        self.add_signal(self.semaphore_wait_signal_class, |signal_object| {
            signal_object.inst_var_at_put(2, signal.semaphore);

            signal_object.inst_var_at_put(
                3,
                RawObjectPointer::new(Smalltalk::primitive_bool_object(signal.is_locked).as_i64()),
            );
        });
    }

    fn add_signal(&mut self, signal_class: ObjectRef, callback: impl FnOnce(&mut Object)) {
        let signal_pointer = Smalltalk::primitive_instantiate_class(
            ObjectPointer::from(signal_class.into_inner().as_i64()),
            false,
        );
        let mut signal_pointer = AnyObjectRef::from(RawObjectPointer::new(signal_pointer.as_i64()));

        let mut signal_object_ref = signal_pointer.as_object().unwrap();
        let signal_object = signal_object_ref.deref_mut();

        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        signal_object.inst_var_at_put(0, Immediate::new_i64(since_the_epoch.as_secs() as i64));

        signal_object.inst_var_at_put(
            1,
            Immediate::new_i64(since_the_epoch.subsec_nanos() as i64),
        );

        callback(signal_object);

        //self.signals.add_last(signal_object);
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct LocalProcessSwitchTelemetryRef(ObjectRef);

impl AbstractTelemetry for LocalProcessSwitchTelemetryRef {
    fn receive_signal(&mut self, signal: &TelemetrySignal) {
        match signal {
            TelemetrySignal::ContextSwitch(signal) => {
                self.receive_context_switch_signal(signal);
            }
            TelemetrySignal::SemaphoreWait(signal) => {
                self.receive_semaphore_wait_signal(signal);
            }
            TelemetrySignal::ComputationSignal(signal) => {
                self.receive_computation_signal(signal);
            }
            TelemetrySignal::ContextSignal(signal) => {
                self.receive_context_signal(signal);
            }
        }
    }

    fn assign_id(&mut self, id: usize) {
        self.id = Immediate::new_i64(id as i64);
    }
}

impl Deref for LocalProcessSwitchTelemetryRef {
    type Target = LocalProcessSwitchTelemetry;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for LocalProcessSwitchTelemetryRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for LocalProcessSwitchTelemetryRef {
    type Error = ApplicationError;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;

        if object.amount_of_slots() != 5 {
            return Err(vm_object_model::Error::InvalidType(
                "LocalProcessSwitchTelemetry".to_string(),
            )
            .into());
        }

        Ok(Self(object))
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStartLocalProcessSwitchTelemetry() {
    let telemetry_pointer = Smalltalk::stack_ref(StackOffset::new(0));
    match LocalProcessSwitchTelemetryRef::try_from(telemetry_pointer) {
        Ok(telemetry) => {
            GlobalTelemetry::register(telemetry);
            Smalltalk::method_return_boolean(true);
        }
        Err(error) => {
            error!("Failed to convert stack ref to object: {}", error);
            Smalltalk::primitive_fail();
        }
    }
}
