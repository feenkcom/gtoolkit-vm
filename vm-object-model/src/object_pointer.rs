use std::ffi::c_void;
use std::ptr::{with_exposed_provenance, with_exposed_provenance_mut};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
