use crate::assign_field;
use std::fmt::Debug;
use std::ops::Deref;
use vm_bindings::Smalltalk;
use vm_object_model::{AnyObjectRef, Object, ObjectRef, Result};

#[derive(Debug, PharoObject)]
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
        assign_field!(self.key, key.into());
    }

    pub fn set_value(&mut self, value: impl Into<AnyObjectRef>) {
        assign_field!(self.value, value.into());
    }
}
