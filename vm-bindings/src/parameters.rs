use crate::bindings::{
    vm_parameters_destroy, vm_parameters_parse, VMParameterVector as NativeVMParameterVector,
    VMParameters as NativeVMParameters,
};
use crate::prelude::{Handle, NativeAccess, NativeDrop};
use std::ffi::{CStr, CString};
use std::fmt;
use std::mem::forget;
use std::os::raw::{c_char, c_int, c_void};

pub type VMParameters = Handle<NativeVMParameters>;

impl NativeDrop for NativeVMParameters {
    fn drop(&mut self) {
        unsafe {
            vm_parameters_destroy(self);
        }
    }
}

impl VMParameters {
    pub fn from_args(arguments: Vec<String>) -> Self {
        // create a vector of zero terminated strings
        let args = arguments
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect::<Vec<CString>>();

        // convert the strings to raw pointers
        let mut c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let vars = std::env::vars()
            .map(|arg| CString::new(format!("{}{}", arg.0, arg.1)).unwrap())
            .collect::<Vec<CString>>();

        let mut c_vars = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut default_parameters = Self::default();
        default_parameters.native_mut().processArgc = c_args.len() as c_int;
        default_parameters.native_mut().processArgv = c_args.as_mut_ptr();
        default_parameters.native_mut().environmentVector = c_vars.as_mut_ptr();

        unsafe {
            vm_parameters_parse(
                c_args.len() as c_int,
                c_args.as_mut_ptr(),
                default_parameters.native_mut(),
            )
        };
        // "leak" the args, since the memory is handled by parameters now
        forget(args);
        forget(c_args);
        forget(vars);
        forget(c_vars);

        default_parameters
    }

    pub fn from_env_args() -> Self {
        Self::from_args(std::env::args().collect())
    }

    pub fn image_file_name(&self) -> String {
        let c_str: &CStr = unsafe { CStr::from_ptr(self.native().imageFileName) };
        let str_slice: &str = c_str.to_str().unwrap();
        str_slice.to_owned()
    }

    pub fn set_image_file_name(&mut self, file_name: String) {
        if self.image_file_name() == file_name {
            return;
        }

        let previous_file_name = self.native().imageFileName as *mut c_void;
        unsafe { crate::bindings::free(previous_file_name) };

        let c_str = CString::new(file_name).unwrap();
        self.native_mut().imageFileName = c_str.into_raw();
        self.native_mut().isDefaultImage = false;
    }

    pub fn is_default_image(&self) -> bool {
        self.native().isDefaultImage
    }

    pub fn is_default_image_found(&self) -> bool {
        self.native().defaultImageFound
    }

    pub fn is_interactive_session(&self) -> bool {
        self.native().isInteractiveSession
    }

    pub fn set_is_interactive_session(&mut self, is_interactive_session: bool) {
        self.native_mut().isInteractiveSession = is_interactive_session;
    }

    pub fn max_stack_frames_to_print(&self) -> usize {
        self.native().maxStackFramesToPrint as usize
    }

    pub fn max_old_space_size(&self) -> usize {
        self.native().maxOldSpaceSize as usize
    }

    pub fn max_code_size(&self) -> usize {
        self.native().maxCodeSize as usize
    }

    pub fn eden_size(&self) -> usize {
        self.native().edenSize as usize
    }
}

impl fmt::Debug for VMParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VMParameters")
            .field("image_file_name", &self.image_file_name())
            .field("is_default_image", &self.is_default_image())
            .field("is_default_image_found", &self.is_default_image_found())
            .field("is_interactive_session", &self.is_interactive_session())
            .field(
                "max_stack_frames_to_print",
                &self.max_stack_frames_to_print(),
            )
            .field("max_old_space_size", &self.max_old_space_size())
            .field("max_code_size", &self.max_code_size())
            .field("eden_size", &self.eden_size())
            .finish()
    }
}

impl Default for VMParameters {
    fn default() -> Self {
        Self::from_native_c(NativeVMParameters {
            imageFileName: std::ptr::null_mut(),
            isDefaultImage: false,
            defaultImageFound: false,
            isInteractiveSession: false,
            maxStackFramesToPrint: 0,
            maxOldSpaceSize: 0,
            maxCodeSize: 0,
            edenSize: 0,
            processArgc: 0,
            processArgv: std::ptr::null_mut(),
            environmentVector: std::ptr::null_mut(),
            vmParameters: NativeVMParameterVector {
                count: 0,
                parameters: std::ptr::null_mut(),
            },
            imageParameters: NativeVMParameterVector {
                count: 0,
                parameters: std::ptr::null_mut(),
            },
        })
    }
}
