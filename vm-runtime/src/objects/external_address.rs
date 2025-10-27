use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::slice;
use vm_object_model::{AnyObjectRef, Error, Object, ObjectFormat, ObjectRef, Result};

#[derive(Debug)]
#[repr(C)]
pub struct ExternalAddress {
    this: Object,
}

impl ExternalAddress {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.first_fixed_field_ptr() as _, self.len()) }
    }

    pub fn len(&self) -> usize {
        self.amount_of_indexable_units()
    }
    
    pub fn read_address(&self) -> *const c_void {
        let ptr = self.first_fixed_field_ptr() as *const *const c_void;
        unsafe { *ptr }
    }
    
    pub fn set_address(&mut self, address: *const c_void) {
        let ptr = self.first_fixed_field_ptr() as *mut *const c_void;
        unsafe { *ptr = address };
    }
    
    pub fn is_null(&self) -> bool {
        self.read_address().is_null()
    }
}

impl Deref for ExternalAddress {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ExternalAddressRef(ObjectRef);

impl Deref for ExternalAddressRef {
    type Target = ExternalAddress;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for ExternalAddressRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for ExternalAddressRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;
        match object.object_format() {
            ObjectFormat::Indexable8(_) => Ok(Self(object)),
            _ => Err(Error::InvalidType("ExternalAddress".to_string())),
        }
    }
}

impl From<ExternalAddressRef> for AnyObjectRef {
    fn from(value: ExternalAddressRef) -> Self {
        value.0.into()
    }
}
