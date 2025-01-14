use crate::object::Object;
use crate::objects::{hash_of, ArrayRef};
use crate::Result;
use crate::{AnyObjectRef, Error, ObjectRef};
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug)]
#[repr(C)]
pub struct WeakSymbolSet {
    this: Object,
    tally: AnyObjectRef,
    array: ArrayRef,
    flag: ObjectRef,
}

impl WeakSymbolSet {
    pub fn find_like_byte_str(&self, string: &str) -> Option<ObjectRef> {
        if let Some(index) = self.scan_for_byte_str(string) {
            let item = self.array.get(index)?.as_object().ok()?;

            if !item.equals(&self.flag).ok()? {
                return Some(item);
            }
        }
        None
    }

    pub fn scan_for_byte_str(&self, string: &str) -> Option<usize> {
        let hash: u32 = hash_of(string);

        let start = hash.rem_euclid(self.array.len() as u32) as usize;
        let mut index = start;

        loop {
            if let Ok(item) = self.array[index].as_object() {
                if item.equals(&self.flag).ok()? {
                    return Some(index);
                }

                if let Some(byte_symbol) = item.try_as_byte_symbol() {
                    if byte_symbol.as_str() == string {
                        return Some(index);
                    }
                }
            }

            index = index.rem_euclid(self.array.len());
            if index == start {
                break;
            }
        }

        None
    }
}

impl Deref for WeakSymbolSet {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct WeakSymbolSetRef(ObjectRef);

impl Deref for WeakSymbolSetRef {
    type Target = WeakSymbolSet;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl TryFrom<AnyObjectRef> for WeakSymbolSetRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;

        if object.amount_of_slots() != 3 {
            return Err(Error::InvalidType("WeakSymbolSet".to_string()));
        }

        Ok(Self(object))
    }
}
