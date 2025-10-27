use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::slice;
use vm_object_model::{AnyObjectRef, Error, Object, ObjectFormat, ObjectRef, Result};

#[derive(Debug)]
#[repr(C)]
pub struct ByteArray {
    this: Object,
}

impl ByteArray {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.first_fixed_field_ptr() as _, self.len()) }
    }
    
    pub fn len(&self) -> usize {
        self.amount_of_indexable_units()
    }
}

impl Deref for ByteArray {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ByteArrayRef(ObjectRef);

impl Deref for ByteArrayRef {
    type Target = ByteArray;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for ByteArrayRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for ByteArrayRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;
        match object.object_format() {
            ObjectFormat::Indexable8(_) => Ok(Self(object)),
            _ => Err(Error::InvalidType("ByteSymbol".to_string())),
        }
    }
}

impl From<ByteArrayRef> for AnyObjectRef {
    fn from(value: ByteArrayRef) -> Self {
        value.0.into()
    }
}
