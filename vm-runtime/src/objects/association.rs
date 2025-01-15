use log::__private_api::Value;
use std::ops::{Deref, DerefMut};
use vm_bindings::Smalltalk;
use vm_object_model::{AnyObjectRef, Error, Object, ObjectRef, Result};

#[derive(Debug)]
#[repr(C)]
pub struct Association {
    this: Object,
    key: AnyObjectRef,
    value: AnyObjectRef,
}

impl Association {
    pub fn new(association_class: ObjectRef) -> Result<AssociationRef> {
        Smalltalk::instantiate(association_class)
    }

    pub fn key(&self) -> AnyObjectRef {
        self.key
    }

    pub fn value(&self) -> AnyObjectRef {
        self.value
    }

    pub fn set_key(&mut self, key: impl Into<AnyObjectRef>) {
        self.key = key.into();
    }

    pub fn set_value(&mut self, value: impl Into<AnyObjectRef>) {
        self.value = value.into();
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct AssociationRef(ObjectRef);

impl Deref for AssociationRef {
    type Target = Association;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for AssociationRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for AssociationRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 2;
        let object = value.as_object()?;
        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object_ref: object,
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<AssociationRef> for AnyObjectRef {
    fn from(value: AssociationRef) -> Self {
        value.0.into()
    }
}
