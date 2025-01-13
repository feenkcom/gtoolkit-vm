use crate::{AnyObject, ObjectFormat, ObjectHeader, RawObjectPointer};
use std::fmt::Debug;
use std::os::raw::c_void;

#[derive(Clone)]
pub struct Array<'image> {
    header: &'image ObjectHeader,
    items: &'image [RawObjectPointer],
}

impl<'obj> Array<'obj> {
    pub fn new(header: &'obj ObjectHeader, items: &'obj [RawObjectPointer]) -> Self {
        Self { header, items }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.header as *const _ as *const c_void
    }

    pub fn header(&self) -> &'obj ObjectHeader {
        self.header
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn raw_items(&self) -> &'obj [RawObjectPointer] {
        self.items
    }

    pub fn items(&self) -> impl Iterator<Item = AnyObject> {
        self.items.iter().map(|each| each.reify())
    }

    pub fn get(&self, index: usize) -> Option<AnyObject> {
        self.items.get(index).map(|each| each.reify())
    }
}

pub struct ArrayMut<'image> {
    header: &'image ObjectHeader,
    items: &'image mut [RawObjectPointer],
}

impl<'obj> ArrayMut<'obj> {
    pub fn new(header: &'obj ObjectHeader, items: &'obj mut [RawObjectPointer]) -> Self {
        Self { header, items }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.header as *const _ as *const c_void
    }

    pub fn header(&self) -> &'obj ObjectHeader {
        self.header
    }

    pub fn raw_header(&self) -> RawObjectPointer {
        RawObjectPointer::from(self.as_ptr())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> impl Iterator<Item = AnyObject> {
        self.items.iter().map(|each| each.reify())
    }

    pub fn insert(&mut self, index: usize, object: &AnyObject) {
        self.items[index] = object.raw_header();
    }
}

impl Debug for Array<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Array")
            .field("header", &self.header)
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

impl Debug for ArrayMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayMut")
            .field("header", &self.header)
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ArrayPointer(RawObjectPointer);

impl ArrayPointer {
    pub fn as_array(&self) -> Array {
        let object = self.0.reify().as_object_unchecked();
        let length = object.len();
        let slice_ptr = object.first_fixed_field_ptr() as *const RawObjectPointer;

        let slice: &[RawObjectPointer] =
            unsafe { std::slice::from_raw_parts(slice_ptr, length) };

        Array::new(object.header(), slice)
    }
}

impl TryFrom<RawObjectPointer> for ArrayPointer {
    type Error = String;

    fn try_from(pointer: RawObjectPointer) -> Result<Self, Self::Error> {
        let object = pointer.reify();
        if let Some(object) = object.try_as_object() {
            match object.object_format() {
                ObjectFormat::IndexableWithoutInstVars | ObjectFormat::WeakIndexable => {
                    Ok(Self(pointer))
                }
                _ => Err(format!("RawPointer is not an Array: {:?}", object)),
            }
        }
        else {
            Err("RawPointer is not an object".into())
        }
    }
}
