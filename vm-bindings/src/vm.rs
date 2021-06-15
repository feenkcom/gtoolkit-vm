use crate::bindings::vm_main_with_parameters;
use crate::prelude::NativeAccess;
use crate::VMParameters;
use std::error::Error;

pub struct VM {}

impl VM {
    pub fn start(mut parameters: VMParameters) -> Result<(), Box<dyn Error>> {
        unsafe {
            vm_main_with_parameters(parameters.native_mut());
        }

        Ok(())
    }
}
