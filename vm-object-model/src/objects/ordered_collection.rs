use crate::objects::ArrayRef;
use crate::{AnyObjectRef, Error, Immediate, Object, ObjectRef, Result};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
#[repr(C)]
pub struct OrderedCollection {
    this: Object,
    array: ArrayRef,
    first_index: Immediate,
    last_index: Immediate,
}

impl OrderedCollection {
    pub fn add_last(&mut self, object: impl Into<AnyObjectRef>) {
        let last_index = self.last_index.as_integer().unwrap();
        self.array.insert(last_index as usize, object);
        self.last_index = Immediate::new_integer(last_index + 1);
    }
}

impl Deref for OrderedCollection {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct OrderedCollectionRef(ObjectRef);

impl Deref for OrderedCollectionRef {
    type Target = OrderedCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for OrderedCollectionRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for OrderedCollectionRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;

        if object.amount_of_slots() != 3 {
            return Err(Error::InvalidType("OrderedCollection".to_string()));
        }

        Ok(Self(object))
    }
}
