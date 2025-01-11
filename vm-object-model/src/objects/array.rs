use crate::{AnyObject, ObjectHeader, RawObjectPointer};
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
}

impl Debug for Array<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Array")
            .field("header", &self.header)
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}
