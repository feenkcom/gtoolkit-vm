use crate::bindings::VMParameterVector as NativeVMParameterVector;
use crate::prelude::{transmute_ref, NativeAccess};
use std::ffi::CStr;

#[repr(transparent)]
pub struct VirtualMachineParameters(NativeVMParameterVector);

impl NativeAccess<NativeVMParameterVector> for VirtualMachineParameters {
    fn native(&self) -> &NativeVMParameterVector {
        &self.0
    }

    fn native_mut(&mut self) -> &mut NativeVMParameterVector {
        &mut self.0
    }
}

impl VirtualMachineParameters {
    pub fn len(&self) -> usize {
        self.native().count as usize
    }

    pub fn iter(&self) -> ParametersVectorIterator {
        ParametersVectorIterator {
            vector: self.native(),
            index: 0,
        }
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.iter().collect()
    }

    pub(crate) fn borrow_from_native(native: &NativeVMParameterVector) -> &Self {
        unsafe { transmute_ref(native) }
    }
}

#[repr(transparent)]
pub struct ImageParameters(NativeVMParameterVector);

impl NativeAccess<NativeVMParameterVector> for ImageParameters {
    fn native(&self) -> &NativeVMParameterVector {
        &self.0
    }

    fn native_mut(&mut self) -> &mut NativeVMParameterVector {
        &mut self.0
    }
}

impl ImageParameters {
    pub fn len(&self) -> usize {
        self.native().count as usize
    }

    pub fn iter(&self) -> ParametersVectorIterator {
        ParametersVectorIterator {
            vector: self.native(),
            index: 0,
        }
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.iter().collect()
    }

    pub(crate) fn borrow_from_native(native: &NativeVMParameterVector) -> &Self {
        unsafe { transmute_ref(native) }
    }
}

pub struct ParametersVectorIterator<'a> {
    vector: &'a NativeVMParameterVector,
    index: usize,
}

impl<'a> Iterator for ParametersVectorIterator<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.index < self.vector.count as usize {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.vector.parameters, self.vector.count as usize)
            };
            let chars = unsafe { CStr::from_ptr(slice[self.index]) };
            let string = String::from(chars.to_string_lossy());
            self.index += 1;
            Some(string)
        } else {
            None
        }
    }
}
