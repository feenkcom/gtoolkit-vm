use crate::object::Object;
use crate::{AnyObject, ObjectFormat, ObjectHeader};
use std::ffi::c_void;

#[derive(Debug)]
pub struct CompiledMethod<'obj> {
    header: &'obj Object,
}

impl<'obj> CompiledMethod<'obj> {
    pub fn set_literal(&self, literal: AnyObject<'obj>, literal_index: usize) {
        let compiled_method_header = self.header.first_fixed_field_ptr();
        let len = self.header.len();

        let mut literal_ptr =
            unsafe { compiled_method_header.offset((1 + literal_index as isize) * 8) }
                as *mut *const c_void;
        unsafe { *literal_ptr = literal.as_ptr() };
    }
}

impl<'obj> TryFrom<&'obj Object> for CompiledMethod<'obj> {
    type Error = String;

    fn try_from(value: &'obj Object) -> Result<Self, Self::Error> {
        match value.object_format() {
            ObjectFormat::CompiledMethod(_) => Ok(CompiledMethod { header: value }),
            _ => Err("Object is not compiled method".into()),
        }
    }
}
