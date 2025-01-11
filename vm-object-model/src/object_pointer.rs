use crate::object::Object;
use crate::ObjectHeader;
use std::ffi::c_void;
use std::mem::transmute;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct RawObjectPointer(i64);

impl RawObjectPointer {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn reify(&self) -> AnyObject {
        AnyObject::from(self.0)
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

impl From<*const c_void> for RawObjectPointer {
    fn from(value: *const c_void) -> Self {
        unsafe { transmute(value) }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AnyObject<'obj> {
    Immediate(Immediate),
    Object(&'obj Object),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Immediate(pub i64);

impl Immediate {
    pub fn as_ptr(&self) -> *const c_void {
        unsafe { transmute(self.0) }
    }
}

impl<'obj> AnyObject<'obj> {
    pub fn as_ptr(&self) -> *const c_void {
        match self {
            AnyObject::Immediate(value) => { value.as_ptr() }
            AnyObject::Object(obj) => { obj.as_ptr() }
        }
    }

    pub fn try_as_object(&self) -> Option<&'obj Object> {
        match self {
            AnyObject::Immediate(_) => None,
            AnyObject::Object(object) => Some(object),
        }
    }

    pub fn as_object_unchecked(&self) -> &'obj Object {
        match self {
            AnyObject::Immediate(_) => {
                panic!("Attempted to convert immediate value to object")
            }
            AnyObject::Object(object) => object,
        }
    }

    pub fn is_identical(&self, second: &Self) -> Option<bool> {
        match self {
            Self::Immediate(first) => match second {
                Self::Immediate(second) => Some(first == second),
                Self::Object(_) => Some(false),
            },
            Self::Object(first) => match second {
                Self::Immediate(_) => Some(false),
                Self::Object(second) => first.is_identical(second),
            },
        }
    }
}

impl From<i64> for AnyObject<'_> {
    fn from(value: i64) -> Self {
        if is_immediate(value) {
            AnyObject::Immediate(Immediate(value))
        } else {
            let raw_header: *mut ObjectHeader = unsafe { transmute(value) };
            let object_header = unsafe { &*raw_header };
            let object = unsafe { transmute(object_header) };
            AnyObject::Object(object)
        }
    }
}
