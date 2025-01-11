use crate::object::Object;
use crate::objects::{hash_of, Array};
use crate::{AnyObject, ObjectHeader};
use std::hash::Hash;

#[derive(Debug)]
pub struct WeakSymbolSet<'image> {
    header: &'image ObjectHeader,
    tally: AnyObject<'image>,
    array: Array<'image>,
    flag: AnyObject<'image>,
}

impl<'obj> WeakSymbolSet<'obj> {
    pub fn find_like_byte_str(&self, string: &str) -> Option<&Object> {
        if let Some(index) = self.scan_for_byte_str(string) {
            let raw_item = &self.array.raw_items()[index];
            let item = raw_item.reify();

            if !item.is_identical(&self.flag)? {
                return Some(item.as_object_unchecked());
            }
        }
        None
    }

    pub fn scan_for_byte_str(&self, string: &str) -> Option<usize> {
        let hash: u32 = hash_of(string);

        let start = hash.rem_euclid(self.array.len() as u32) as usize;
        let mut index = start;

        loop {
            let raw_item = self.array.raw_items()[index];
            let item = raw_item.reify();

            if item.is_identical(&self.flag)? {
                return Some(index);
            }

            let item = item.as_object_unchecked();

            if let Some(byte_symbol) = item.try_as_byte_symbol() {
                if byte_symbol.as_str() == string {
                    return Some(index);
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

impl<'obj> TryFrom<&'obj Object> for WeakSymbolSet<'obj> {
    type Error = String;

    fn try_from(value: &'obj Object) -> Result<Self, Self::Error> {
        let tally = value
            .inst_var_at(0)
            .ok_or_else(|| "Tally is not defined".to_string())?;
        let array = value
            .inst_var_at(1)
            .and_then(|array| array.try_as_object())
            .and_then(|array| array.try_as_array())
            .ok_or_else(|| "Array is not defined".to_string())?;
        let flag = value
            .inst_var_at(2)
            .ok_or_else(|| "Flag is not defined".to_string())?;

        Ok(WeakSymbolSet {
            header: value.header(),
            tally,
            array,
            flag,
        })
    }
}
