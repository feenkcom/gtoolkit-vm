use crate::objects::{Array, OrderedCollection, OrderedCollectionRef};
use crate::{
    AbstractTelemetry, ApplicationError, ContextSwitchSignal, GlobalTelemetry,
    IdentityDictionaryRef, Result, SemaphoreWaitSignal, TelemetrySignal,
};
use std::ops::{Deref, DerefMut};
use std::time::{SystemTime, UNIX_EPOCH};
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[repr(C)]
pub struct GlobalProcessSwitchTelemetry {
    this: Object,
    id: Immediate,
    start_time: ObjectRef,
    signals_dictionary: IdentityDictionaryRef,
    signals_collection: OrderedCollectionRef,
    context_switch_signal_class: ObjectRef,
    semaphore_wait_signal_class: ObjectRef,
    ordered_collection_class: ObjectRef,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct GlobalProcessSwitchTelemetryRef(ObjectRef);

impl GlobalProcessSwitchTelemetryRef {
    fn receive_context_switch_signal(&mut self, signal: &ContextSwitchSignal) {
        self.add_context_switch_signal(signal.old_process, false);
        self.add_context_switch_signal(signal.new_process, true);
    }

    fn receive_semaphore_wait_signal(&mut self, signal: &SemaphoreWaitSignal) {
        if signal.is_locked {
            self.add_semaphore_wait_signal(signal.process, signal);
        }
    }

    fn add_context_switch_signal(&mut self, process: ObjectRef, alive: bool) {
        self.add_signal(process, self.context_switch_signal_class, |signal_object| {
            signal_object.inst_var_at_put(
                2,
                RawObjectPointer::new(Smalltalk::bool_object(alive).as_i64()),
            );
        });
    }

    fn add_semaphore_wait_signal(&mut self, process: ObjectRef, signal: &SemaphoreWaitSignal) {
        self.add_signal(process, self.semaphore_wait_signal_class, |signal_object| {
            signal_object.inst_var_at_put(2, signal.semaphore);

            signal_object.inst_var_at_put(
                3,
                RawObjectPointer::new(Smalltalk::bool_object(signal.is_locked).as_i64()),
            );
        });
    }

    fn add_signal(
        &mut self,
        process: ObjectRef,
        signal_class: ObjectRef,
        callback: impl FnOnce(&mut Object),
    ) {
        let signal_pointer = Smalltalk::primitive_instantiate_class(
            ObjectPointer::from(signal_class.into_inner().as_i64()),
            false,
        );
        let mut signal_pointer = AnyObjectRef::from(RawObjectPointer::new(signal_pointer.as_i64()));

        let mut signal_object_ref = signal_pointer.as_object().unwrap();
        let signal_object = signal_object_ref.deref_mut();

        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        signal_object.inst_var_at_put(0, Immediate::new_integer(since_the_epoch.as_secs() as i64));

        signal_object.inst_var_at_put(
            1,
            Immediate::new_integer(since_the_epoch.subsec_nanos() as i64),
        );

        callback(signal_object);

        // self.signals.get_or_insert(process, || {
        //     Array::new(1).unwrap().into()
        //     // OrderedCollection::with_capacity(self.ordered_collection_class, 10)
        //     //     .unwrap()
        //     //     .into()
        // });

        self.signals_collection.add_last(signal_pointer);

        // let mut ordered_collection =
        //     OrderedCollectionRef::try_from(self.signals.get_or_insert(process, || {
        //         OrderedCollection::with_capacity(self.ordered_collection_class, 5000)
        //             .unwrap()
        //             .into()
        //     }))
        //     .unwrap();

        //ordered_collection.add_last(signal_object);
    }
}

impl AbstractTelemetry for GlobalProcessSwitchTelemetryRef {
    fn receive_signal(&mut self, signal: &TelemetrySignal) {
        match signal {
            TelemetrySignal::ContextSwitch(signal) => {
                self.receive_context_switch_signal(signal);
            }
            TelemetrySignal::SemaphoreWait(signal) => {
                self.receive_semaphore_wait_signal(signal);
            }
        }
    }

    fn assign_id(&mut self, id: usize) {
        self.id = Immediate::new_integer(id as i64);
    }
}

impl Deref for GlobalProcessSwitchTelemetryRef {
    type Target = GlobalProcessSwitchTelemetry;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for GlobalProcessSwitchTelemetryRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for GlobalProcessSwitchTelemetryRef {
    type Error = ApplicationError;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 7;
        let object = value.as_object()?;
        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(vm_object_model::Error::WrongAmountOfSlots {
                object_ref: object,
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStartGlobalProcessSwitchTelemetry() {
    let telemetry_pointer = Smalltalk::stack_ref(StackOffset::new(0));
    match GlobalProcessSwitchTelemetryRef::try_from(telemetry_pointer) {
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
