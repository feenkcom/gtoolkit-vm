use crate::objects::{hash_of, ArrayRef, ByteSymbol};
use std::hash::Hash;
use std::ops::Deref;
use vm_object_model::{AnyObjectRef, Object, ObjectRef};

#[derive(Debug, PharoObject)]
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

                if let Ok(byte_symbol) = ByteSymbol::try_from(item.deref()) {
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
