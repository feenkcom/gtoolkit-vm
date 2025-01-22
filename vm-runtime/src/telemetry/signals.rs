use crate::assign_field;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use vm_bindings::Smalltalk;
use vm_object_model::{AnyObjectRef, Error, Immediate, Object, ObjectRef, RawObjectPointer, Result};

#[repr(C)]
pub struct PharoProcessSwitchSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    is_resumed: ObjectRef
}

impl PharoProcessSwitchSignal {
    pub fn set_resumed(&mut self, is_resumed: bool) {
        assign_field!(self.is_resumed, Smalltalk::bool_object(is_resumed));
    }

    pub fn set_timestamp(&mut self, since_the_epoch: Duration) {
        self.seconds = Immediate::new_i64(since_the_epoch.as_secs() as i64);
        self.nanos = Immediate::new_i64(since_the_epoch.subsec_nanos() as i64);
    }
}

#[repr(C)]
pub struct PharoProcessSemaphoreWaitSignal {
    this: Object,
    seconds: Immediate,
    nanos: Immediate,
    semaphore: ObjectRef,
    is_locked: ObjectRef
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
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 3;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
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
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 4;
        let object = value.as_object()?;
        if object.is_forwarded() {
            panic!("Object is forwarded!");
        }

        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object: object.header().clone(),
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
