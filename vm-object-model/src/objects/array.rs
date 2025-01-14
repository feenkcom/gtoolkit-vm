use crate::{AnyObjectRef, Error, Object, ObjectFormat, ObjectRef};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Index};

#[repr(C)]
pub struct Array {
    this: Object,
    items: Items,
}

impl Array {
    pub fn get(&self, index: usize) -> Option<AnyObjectRef> {
        self.as_slice().get(index).map(|item| item.clone())
    }

    pub fn insert(&mut self, index: usize, object: impl Into<AnyObjectRef>) {
        self.as_slice_mut()[index] = object.into();
    }

    pub fn len(&self) -> usize {
        self.this.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnyObjectRef> {
        self.as_slice().iter()
    }

    pub fn as_slice(&self) -> &[AnyObjectRef] {
        let length = self.len();
        let slice_ptr = &self.items as *const _ as *const AnyObjectRef;
        unsafe { std::slice::from_raw_parts(slice_ptr, length) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [AnyObjectRef] {
        let length = self.len();
        let slice_ptr = &mut self.items as *mut _ as *mut AnyObjectRef;
        unsafe { std::slice::from_raw_parts_mut(slice_ptr, length) }
    }
}

impl Index<usize> for Array {
    type Output = AnyObjectRef;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_slice().get(index).unwrap()
    }
}

impl Debug for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Array")
            .field("header", &self.this)
            .field("items", &self.len())
            .finish_non_exhaustive()
    }
}

impl Deref for Array {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[repr(transparent)]
pub struct Items(AnyObjectRef);

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ArrayRef(ObjectRef);

impl Deref for ArrayRef {
    type Target = Array;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for ArrayRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for ArrayRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self, Self::Error> {
        let object_ref = value.as_object()?;
        match object_ref.object_format() {
            ObjectFormat::IndexableWithoutInstVars | ObjectFormat::WeakIndexable => {
                Ok(ArrayRef(object_ref))
            }
            object_format => Err(Error::NotAnArray {
                object_ref,
                object_format,
            }),
        }
    }
}
