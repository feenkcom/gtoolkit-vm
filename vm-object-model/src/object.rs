use std::hash::{Hash, Hasher};
use crate::{Error, Immediate, ObjectFormat, ObjectHeader, RawObjectPointer, Result};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;

#[derive(Debug)]
#[repr(transparent)]
pub struct Object(ObjectHeader);

impl Object {
    const FORWARDED_OBJECT_CLASS_INDEX_PUN: u32 = 8;
    const SHIFT_FOR_WORD: u32 = 3;

    pub fn header(&self) -> &ObjectHeader {
        &self.0
    }

    /// Return a number of slots.
    /// Should not be applied to free or forwarded objects.
    pub fn amount_of_slots(&self) -> usize {
        assert!(
            self.0.class_index() > Self::FORWARDED_OBJECT_CLASS_INDEX_PUN,
            "Must not be free or forwarded object"
        );

        self.amount_of_slots_unchecked()
    }

    /// An unchecked version of `amount_of_slots`: that can be applied to free or forwarded objects.
    pub fn amount_of_slots_unchecked(&self) -> usize {
        let num_slots = self.0.num_slots();
        if num_slots == 255 {
            unsafe {
                let ptr = self.as_ptr().offset(-(size_of::<ObjectHeader>() as isize)) as *mut i64;
                let value = u64::try_from(*ptr << 8).unwrap() >> 8;
                value as usize
            }
        } else {
            num_slots as usize
        }
    }

    pub fn object_format(&self) -> ObjectFormat {
        self.0.format()
    }

    /// Answer the number of indexable units in the object.
    /// For a CompiledMethod, the size of the method header (in bytes)
    /// should be subtracted from the result of this method.
    ///
    /// lengthOf: objOop format: fmt
    pub fn amount_of_indexable_units(&self) -> usize {
        self.object_format()
            .amount_of_indexable_units(self.amount_of_slots_unchecked())
    }

    pub fn as_ptr(&self) -> *const c_void {
        self as *const _ as *const c_void
    }

    /// Return a pointer to the object memory right after the header
    pub fn first_fixed_field_ptr(&self) -> *const c_void {
        unsafe { self.as_ptr().offset(size_of::<ObjectHeader>() as isize) }
    }

    pub fn is_forwarded(&self) -> bool {
        self.0.class_index() <= Self::FORWARDED_OBJECT_CLASS_INDEX_PUN
    }

    pub fn is_identical(&self, second: &Object) -> Option<bool> {
        if self.is_forwarded() {
            return None;
        }

        if second.is_forwarded() {
            return None;
        }

        Some(self.as_ptr() == second.as_ptr())
    }

    pub fn equals(&self, other: &Object) -> Result<bool> {
        // if self.is_forwarded() {
        //     return Err(Error::ForwardedUnsupported(self.header().clone()));
        // }

        if other.is_forwarded() {
            return Err(Error::ForwardedUnsupported(other.header().clone()));
        }

        Ok(self.0 == other.0)
    }

    pub fn inst_var_at(&self, field_index: usize) -> Option<AnyObjectRef> {
        if field_index >= self.amount_of_slots_unchecked() {
            return None;
        }

        let pointer = unsafe {
            self.as_ptr().offset(
                size_of::<ObjectHeader>() as isize + (field_index << Self::SHIFT_FOR_WORD) as isize,
            )
        } as *const i64;

        let pointer_value: i64 = unsafe { *pointer };
        Some(AnyObjectRef::from(RawObjectPointer::from(pointer_value)))
    }

    pub fn inst_var_at_put(&mut self, field_index: usize, object: impl Into<AnyObjectRef>) {
        if field_index >= self.amount_of_slots_unchecked() {
            return;
        }

        let mut pointer = unsafe {
            self.as_ptr().offset(
                size_of::<ObjectHeader>() as isize + (field_index << Self::SHIFT_FOR_WORD) as isize,
            )
        } as *mut i64;

        unsafe { *pointer = object.into().as_i64() };
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct ObjectRef(RawObjectPointer);

impl ObjectRef {
    pub unsafe fn from_raw_pointer_unchecked(pointer: RawObjectPointer) -> Self {
        Self(pointer)
    }
    
    pub fn header(&self) -> &ObjectHeader {
        unsafe { self.cast::<Object>() }.header()
    }

    pub fn is_context(&self) -> bool {
        self.header().class_index() == 36
    }

    pub fn into_inner(self) -> RawObjectPointer {
        self.0
    }

    pub unsafe fn cast<T>(&self) -> &T {
        self.0.cast()
    }

    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        self.0.cast_mut()
    }
}

impl Deref for ObjectRef {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for ObjectRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<RawObjectPointer> for ObjectRef {
    type Error = Error;

    fn try_from(value: RawObjectPointer) -> Result<Self> {
        if !value.is_immediate() {
            Ok(Self(value))
        } else {
            Err(Error::NotAnObject(value))
        }
    }
}

impl From<ObjectRef> for AnyObjectRef {
    fn from(obj: ObjectRef) -> Self {
        Self::from(obj.0)
    }
}

impl From<&Object> for ObjectRef {
    fn from(obj: &Object) -> Self {
        let ptr = obj as *const _ as usize;
        ObjectRef(RawObjectPointer::from(i64::try_from(ptr).unwrap()))
    }
}

impl From<&mut Object> for AnyObjectRef {
    fn from(obj: &mut Object) -> Self {
        let ptr = obj as *mut _ as usize;
        AnyObjectRef::from(RawObjectPointer::from(i64::try_from(ptr).unwrap()))
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

    pub fn amount_of_instance_variables(&self) -> usize {
        self.as_object()
            .map(|object| object.amount_of_slots())
            .unwrap_or(0)
    }

    pub fn amount_of_indexable_units(&self) -> usize {
        self.as_object()
            .map(|object| object.amount_of_indexable_units())
            .unwrap_or(0)
    }

    pub fn as_immediate(&self) -> Result<Immediate> {
        Immediate::try_from(self.0)
    }

    pub fn as_object(&self) -> Result<ObjectRef> {
        ObjectRef::try_from(self.0)
    }
    
    pub fn into_inner(self) -> RawObjectPointer {
        self.0
    }
}

impl From<RawObjectPointer> for AnyObjectRef {
    fn from(value: RawObjectPointer) -> Self {
        Self(value)
    }
}

impl From<Immediate> for AnyObjectRef {
    fn from(immediate: Immediate) -> Self {
        Self::from(RawObjectPointer::from(immediate.0))
    }
}

impl PartialEq<Self> for AnyObjectRef {
    fn eq(&self, other: &Self) -> bool {
        self.as_i64() == other.as_i64()
    }
}

impl Eq for AnyObjectRef {}


impl Hash for AnyObjectRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_i64().hash(state);
    }
}