use crate::objects::{Array, ByteSymbol, WideSymbol};
use crate::{AnyObject, ObjectFormat, ObjectHeader, RawObjectPointer};
use std::os::raw::c_void;
use widestring::U32Str;

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
            "Most not be free or forwarded object"
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
    pub fn len(&self) -> usize {
        self.object_format()
            .amount_of_indexable_units(self.amount_of_slots_unchecked())
    }

    pub fn as_ptr(&self) -> *const c_void {
        unsafe { std::mem::transmute(self) }
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

    pub fn try_as_array(&self) -> Option<Array> {
        match self.object_format() {
            ObjectFormat::IndexableWithoutInstVars | ObjectFormat::WeakIndexable => {
                let length = self.len();
                let slice_ptr = self.first_fixed_field_ptr() as *const RawObjectPointer;

                let slice: &[RawObjectPointer] =
                    unsafe { std::slice::from_raw_parts(slice_ptr, length) };

                Some(Array::new(&self.0, slice))
            }
            _ => None,
        }
    }

    pub fn try_as_byte_symbol(&self) -> Option<ByteSymbol> {
        match self.object_format() {
            ObjectFormat::Indexable8(_) => {
                let length = self.len();
                let slice_ptr = unsafe { self.as_ptr().offset(size_of::<ObjectHeader>() as isize) }
                    as *const u8;

                let source_bytes = unsafe { std::slice::from_raw_parts(slice_ptr, length) };
                let source_str = unsafe { std::str::from_utf8_unchecked(source_bytes) };

                Some(ByteSymbol::new(&self.0, source_str))
            }
            _ => None,
        }
    }

    pub fn try_as_wide_symbol(&self) -> Option<WideSymbol> {
        match self.object_format() {
            ObjectFormat::Indexable32(_) => {
                let length = self.len();
                let slice_ptr = unsafe { self.as_ptr().offset(size_of::<ObjectHeader>() as isize) }
                    as *const u32;

                let source_str = unsafe { U32Str::from_ptr(slice_ptr, length) };

                Some(WideSymbol::new(&self.0, source_str))
            }
            _ => None,
        }
    }

    pub fn inst_var_at(&self, field_index: usize) -> Option<AnyObject> {
        if field_index >= self.amount_of_slots_unchecked() {
            return None;
        }

        let pointer = unsafe {
            self.as_ptr().offset(
                size_of::<ObjectHeader>() as isize + (field_index << Self::SHIFT_FOR_WORD) as isize,
            )
        } as *const i64;

        let pointer_value: i64 = unsafe { *pointer };
        Some(AnyObject::from(pointer_value))
    }
}
