use crate::immediate::Immediate;
use crate::ObjectRef;
use crate::Result;
use std::ffi::c_void;
use std::ptr::{with_exposed_provenance, with_exposed_provenance_mut};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RawObjectPointer(i64);

impl RawObjectPointer {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn as_ptr(&self) -> *const c_void {
        if self.is_immediate() {
            panic!("Can't create a pointer to an immediate object");
        }

        with_exposed_provenance(self.0 as _)
    }

    pub unsafe fn cast<T>(&self) -> &T {
        let ptr: *const T = with_exposed_provenance(self.0 as usize);
        unsafe { &*ptr }
    }

    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        let ptr: *mut T = with_exposed_provenance_mut(self.0 as usize);
        unsafe { &mut *ptr }
    }

    pub fn is_immediate(&self) -> bool {
        is_immediate(self.0)
    }
}

fn is_immediate(value: i64) -> bool {
    pub const TAG_MASK: i64 = 7;

    (value & TAG_MASK) != 0
}

impl From<i64> for RawObjectPointer {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct AnyObjectRef(RawObjectPointer);

impl AnyObjectRef {
    pub fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr()
    }

    pub fn as_i64(&self) -> i64 {
        self.0.as_i64()
    }

    pub fn is_immediate(&self) -> bool {
        self.0.is_immediate()
    }

    pub fn equals(&self, other: &AnyObjectRef) -> Result<bool> {
        if self.is_immediate() {
            if other.is_immediate() {
                Ok(self.0 == other.0)
            } else {
                Ok(false)
            }
        } else {
            if other.is_immediate() {
                Ok(false)
            } else {
                let this = self.as_object()?;
                let other = other.as_object()?;
                this.equals(&other)
            }
        }
    }

    pub fn as_immediate(&self) -> Result<Immediate> {
        Immediate::try_from(self.0)
    }

    pub fn as_object(&self) -> Result<ObjectRef> {
        ObjectRef::try_from(self.0)
    }
}

impl From<RawObjectPointer> for AnyObjectRef {
    fn from(value: RawObjectPointer) -> Self {
        Self(value)
    }
}
