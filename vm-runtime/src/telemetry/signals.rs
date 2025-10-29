use crate::assign_field;
use crate::objects::{Array, ArrayRef};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use vm_bindings::Smalltalk;
use vm_object_model::{
    AnyObjectRef, Error, Immediate, Object, ObjectRef, RawObjectPointer, Result,
};

#[repr(C)]
pub struct PharoProcessSwitchSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    is_resumed: ObjectRef,
    stack: ArrayRef,
}

impl PharoProcessSwitchSignal {
    pub fn set_resumed(&mut self, is_resumed: bool) {
        assign_field!(self.is_resumed, Smalltalk::bool_object(is_resumed));
    }

    pub fn set_timestamp(&mut self, since_the_epoch: Duration) {
        self.seconds = Immediate::new_i64(since_the_epoch.as_secs() as i64);
        self.nanos = Immediate::new_i64(since_the_epoch.subsec_nanos() as i64);
    }

    pub fn set_stack(&mut self, stack: ArrayRef) {
        assign_field!(self.stack, stack);
    }
}

#[repr(C)]
pub struct PharoProcessSemaphoreWaitSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    semaphore: ObjectRef,
    is_locked: ObjectRef,
    stack: ArrayRef,
}

impl PharoProcessSemaphoreWaitSignal {
    pub fn set_locked(&mut self, is_locked: bool) {
        assign_field!(self.is_locked, Smalltalk::bool_object(is_locked));
    }

    pub fn set_semaphore(&mut self, semaphore: ObjectRef) {
        assign_field!(self.semaphore, semaphore);
    }

    pub fn set_timestamp(&mut self, since_the_epoch: Duration) {
        self.seconds = Immediate::new_i64(since_the_epoch.as_secs() as i64);
        self.nanos = Immediate::new_i64(since_the_epoch.subsec_nanos() as i64);
    }

    pub fn set_context(&mut self, context: ObjectRef) {
        let array = copy_stack(context);

        assign_field!(self.stack, array);
    }
}

#[repr(C)]
pub struct PharoProcessComputationSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    object: AnyObjectRef,
    is_start: ObjectRef,
}

impl PharoProcessComputationSignal {
    pub fn set_object(&mut self, object: AnyObjectRef) {
        assign_field!(self.object, object);
    }

    pub fn set_is_start(&mut self, is_start: bool) {
        assign_field!(self.is_start, Smalltalk::bool_object(is_start));
    }

    pub fn set_timestamp(&mut self, since_the_epoch: Duration) {
        self.seconds = Immediate::new_i64(since_the_epoch.as_secs() as i64);
        self.nanos = Immediate::new_i64(since_the_epoch.subsec_nanos() as i64);
    }
}

#[repr(C)]
pub struct PharoProcessContextSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    stack: ArrayRef,
    object: AnyObjectRef,
}

impl PharoProcessContextSignal {
    pub fn set_object(&mut self, object: AnyObjectRef) {
        assign_field!(self.object, object);
    }

    pub fn set_context(&mut self, context: ObjectRef) {
        assign_field!(self.stack, copy_stack(context));
    }

    pub fn set_timestamp(&mut self, since_the_epoch: Duration) {
        self.seconds = Immediate::new_i64(since_the_epoch.as_secs() as i64);
        self.nanos = Immediate::new_i64(since_the_epoch.subsec_nanos() as i64);
    }
}

pub fn copy_stack(context: ObjectRef) -> ArrayRef {
    let stack_length = Smalltalk::context_stack_length(context);
    let mut array = Array::new(stack_length).unwrap();

    let nil_object = ObjectRef::try_from(RawObjectPointer::new(
        Smalltalk::primitive_nil_object().as_i64(),
    ))
    .unwrap();

    let mut sender = context;
    let mut index = 0;
    while sender != nil_object {
        array.insert(index, Smalltalk::context_method(sender));
        index += 1;
        sender = Smalltalk::context_sender(sender);
    }
    array
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PharoProcessSwitchSignalRef(ObjectRef);

impl Deref for PharoProcessSwitchSignalRef {
    type Target = PharoProcessSwitchSignal;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for PharoProcessSwitchSignalRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl Deref for PharoProcessSwitchSignal {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

impl TryFrom<AnyObjectRef> for PharoProcessSwitchSignalRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 4;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
                type_name: std::any::type_name::<Self>().to_string(),
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<PharoProcessSwitchSignalRef> for AnyObjectRef {
    fn from(value: PharoProcessSwitchSignalRef) -> Self {
        value.0.into()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PharoProcessComputationSignalRef(ObjectRef);

impl Deref for PharoProcessComputationSignalRef {
    type Target = PharoProcessComputationSignal;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for PharoProcessComputationSignalRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl Deref for PharoProcessComputationSignal {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

impl TryFrom<AnyObjectRef> for PharoProcessComputationSignalRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 4;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
                type_name: std::any::type_name::<Self>().to_string(),
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<PharoProcessComputationSignalRef> for AnyObjectRef {
    fn from(value: PharoProcessComputationSignalRef) -> Self {
        value.0.into()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PharoProcessContextSignalRef(ObjectRef);

impl Deref for PharoProcessContextSignalRef {
    type Target = PharoProcessContextSignal;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for PharoProcessContextSignalRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl Deref for PharoProcessContextSignal {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

impl TryFrom<AnyObjectRef> for PharoProcessContextSignalRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 4;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
                type_name: std::any::type_name::<Self>().to_string(),
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<PharoProcessContextSignalRef> for AnyObjectRef {
    fn from(value: PharoProcessContextSignalRef) -> Self {
        value.0.into()
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PharoProcessSemaphoreWaitSignalRef(ObjectRef);

impl Deref for PharoProcessSemaphoreWaitSignalRef {
    type Target = PharoProcessSemaphoreWaitSignal;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for PharoProcessSemaphoreWaitSignalRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl Deref for PharoProcessSemaphoreWaitSignal {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

impl TryFrom<AnyObjectRef> for PharoProcessSemaphoreWaitSignalRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 5;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
                type_name: std::any::type_name::<Self>().to_string(),
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<PharoProcessSemaphoreWaitSignalRef> for AnyObjectRef {
    fn from(value: PharoProcessSemaphoreWaitSignalRef) -> Self {
        value.0.into()
    }
}
