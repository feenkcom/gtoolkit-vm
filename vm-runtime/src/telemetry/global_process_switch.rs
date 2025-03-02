use crate::objects::{Array, ArrayRef, OrderedCollection, OrderedCollectionRef};
use crate::{copy_stack, AbstractTelemetry, ApplicationError, ContextSwitchSignal, GlobalTelemetry, IdentityDictionaryRef, ComputationSignal, PharoProcessComputationSignalRef, PharoProcessSemaphoreWaitSignalRef, PharoProcessSwitchSignal, PharoProcessSwitchSignalRef, Result, SemaphoreWaitSignal, TelemetrySignal, ContextSignal, PharoProcessContextSignal, PharoProcessContextSignalRef};
use std::ops::{Deref, DerefMut};
use std::time::{SystemTime, UNIX_EPOCH};
use vm_bindings::{ObjectPointer, Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, Error, Immediate, Object, ObjectRef, RawObjectPointer};

#[derive(Debug)]
#[repr(C)]
pub struct GlobalProcessSwitchTelemetry {
    this: Object,
    id: Immediate,
    start_time: ObjectRef,
    signals_dictionary: IdentityDictionaryRef,
    context_switch_signal_class: ObjectRef,
    semaphore_wait_signal_class: ObjectRef,
    computation_signal_class: ObjectRef,
    context_signal_class: ObjectRef,
    ordered_collection_class: ObjectRef,
}

impl GlobalProcessSwitchTelemetry {
    pub fn validate_non_forward(&self) {
        if self.signals_dictionary.is_forwarded() {
            panic!("It is forwarded!");
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct GlobalProcessSwitchTelemetryRef(ObjectRef);

impl GlobalProcessSwitchTelemetryRef {
    fn receive_context_switch_signal(&mut self, signal: &ContextSwitchSignal) {
        let stack = copy_stack(Smalltalk::this_context());
        self.add_context_switch_signal(signal.old_process, false, stack);
        self.add_context_switch_signal(signal.new_process, true, stack);
    }

    fn receive_semaphore_wait_signal(&mut self, signal: &SemaphoreWaitSignal) {
        if signal.is_locked {
            self.add_semaphore_wait_signal(signal.process, signal);
        }
    }

    fn receive_computation_signal(&mut self, signal: &ComputationSignal) {
        self.add_computation_signal(signal.process, signal.object, signal.is_start);
    }

    fn receive_context_signal(&mut self, signal: &ContextSignal) {
        self.add_context_signal(signal.process, signal.signal);
    }

    fn add_context_switch_signal(&mut self, process: ObjectRef, alive: bool, stack: ArrayRef) {
        self.add_signal::<PharoProcessSwitchSignalRef>(process, self.context_switch_signal_class, |signal_object| {
            signal_object.set_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
            signal_object.set_resumed(alive);
            signal_object.set_stack(stack);
        });
    }



    fn add_semaphore_wait_signal(&mut self, process: ObjectRef, signal: &SemaphoreWaitSignal) {
        self.add_signal::<PharoProcessSemaphoreWaitSignalRef>(process, self.semaphore_wait_signal_class, |signal_object| {
            signal_object.set_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
            signal_object.set_locked(signal.is_locked);
            signal_object.set_semaphore(signal.semaphore);
            signal_object.set_context(Smalltalk::this_context());
        });
    }

    fn add_computation_signal(&mut self, process: ObjectRef, object: AnyObjectRef, is_start: bool) {
        self.add_signal::<PharoProcessComputationSignalRef>(process, self.computation_signal_class, |signal_object| {
            signal_object.set_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
            signal_object.set_object(object);
            signal_object.set_is_start(is_start);
        });
    }

    fn add_context_signal(&mut self, process: ObjectRef, object: AnyObjectRef) {
        self.add_signal::<PharoProcessContextSignalRef>(process, self.context_signal_class, |signal_object| {
            signal_object.set_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
            signal_object.set_object(object);
            signal_object.set_context(Smalltalk::this_context());
        });
    }

    fn add_signal<T: TryFrom<AnyObjectRef, Error = Error> + Into<AnyObjectRef>>(
        &mut self,
        process: ObjectRef,
        signal_class: ObjectRef,
        callback: impl FnOnce(&mut T),
    ) {

        let mut signal = Smalltalk::instantiate::<T>(signal_class).unwrap();

        callback(&mut signal);

        let ordered_collection_class = self.ordered_collection_class;
        let mut ordered_collection = OrderedCollectionRef::try_from(self.signals_dictionary.get_or_insert(process, || {
            OrderedCollection::with_capacity(ordered_collection_class, 10)
                .unwrap()
                .into()
        })).unwrap();

        ordered_collection.add_last(signal);
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

impl Deref for GlobalProcessSwitchTelemetryRef {
    type Target = GlobalProcessSwitchTelemetry;
    fn deref(&self) -> &Self::Target {
        let c: &GlobalProcessSwitchTelemetry = unsafe { self.0.cast() };
        c.validate_non_forward();
        c
    }
}

impl DerefMut for GlobalProcessSwitchTelemetryRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let c: &mut GlobalProcessSwitchTelemetry = unsafe { self.0.cast_mut() };
        c.validate_non_forward();
        c
    }
}

impl TryFrom<AnyObjectRef> for GlobalProcessSwitchTelemetryRef {
    type Error = ApplicationError;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 8;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("It is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(vm_object_model::Error::WrongAmountOfSlots {
                object: object.header().clone(),
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
