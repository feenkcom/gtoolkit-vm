use crate::object::Object;
use crate::ObjectHeader;
use std::ffi::c_void;
use std::mem::transmute;
use crate::immediate::Immediate;

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

    pub fn reify(&self) -> AnyObject {
        AnyObject::from(self.0)
    }

    pub fn reify_mut(&mut self) -> AnyObjectMut {
        AnyObjectMut::from(self.0)
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

impl<'obj> AnyObject<'obj> {
    pub fn as_ptr(&self) -> *const c_void {
        match self {
            Self::Immediate(value) => value.as_ptr(),
            Self::Object(obj) => obj.as_ptr(),
        }
    }

    pub fn raw_header(&self) -> RawObjectPointer {
        RawObjectPointer::from(self.as_ptr())
    }

    pub fn try_as_object(&self) -> Option<&'obj Object> {
        match self {
            Self::Immediate(_) => None,
            Self::Object(object) => Some(object),
        }
    }

    pub fn as_object_unchecked(&self) -> &'obj Object {
        match self {
            Self::Immediate(_) => {
                panic!("Attempted to convert immediate value to object")
            }
            Self::Object(object) => object,
        }
    }

    pub fn as_immediate_unchecked(&self) -> Immediate {
        match self {
            Self::Immediate(value) => *value,
            Self::Object(_) => panic!("Attempted to convert an object to an immediate value"),
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

#[derive(Debug)]
pub enum AnyObjectMut<'obj> {
    Immediate(Immediate),
    Object(&'obj mut Object),
}

impl<'image> AnyObjectMut<'image> {
    pub fn as_ptr(&self) -> *mut c_void {
        match self {
            Self::Immediate(value) => value.as_ptr() as *mut c_void,
            Self::Object(obj) => obj.as_ptr() as *mut c_void,
        }
    }

    pub fn raw_header(&self) -> RawObjectPointer {
        RawObjectPointer::from(self.as_ptr() as *const c_void)
    }

    pub fn is_object(&self) -> bool {
        match self {
            Self::Immediate(_) => false,
            Self::Object(_) => true
        }
    }

    pub fn amount_of_slots(&self) -> usize {
        match self {
            Self::Immediate(_) => 0,
            Self::Object(object) => object.amount_of_slots()
        }
    }

    pub fn try_as_object_mut(&'image mut self) -> Option<&'image mut Object> {
        match self {
            Self::Immediate(_) => None,
            Self::Object(object) => Some(object),
        }
    }

    pub fn try_into_object_mut(mut self) -> Option<&'image mut Object> {
        match self {
            Self::Immediate(_) => None,
            Self::Object(object) => Some(object),
        }
    }

    pub fn as_object_unchecked_mut(&'image mut self) -> &'image mut Object {
        match self {
            Self::Immediate(_) => {
                panic!("Attempted to convert immediate value to object")
            }
            Self::Object(object) => object,
        }
    }

    pub fn into_object_unchecked_mut(mut self) -> &'image mut Object {
        match self {
            Self::Immediate(_) => {
                panic!("Attempted to convert immediate value to object")
            }
            Self::Object(object) => object,
        }
    }
}

impl From<i64> for AnyObjectMut<'_> {
    fn from(value: i64) -> Self {
        if is_immediate(value) {
            AnyObjectMut::Immediate(Immediate(value))
        } else {
            let raw_header: *mut ObjectHeader = unsafe { transmute(value) };
            let object_header = unsafe { &mut *raw_header };
            let object = unsafe { transmute(object_header) };
            AnyObjectMut::Object(object)
        }
    }
}
