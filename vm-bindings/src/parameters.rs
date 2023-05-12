use crate::bindings::{
    free, vm_parameters_destroy, vm_parameters_parse, VMParameterVector as NativeVMParameterVector,
    VMParameters as NativeVMParameters,
};
use crate::parameter_vector::{ImageParameters, VirtualMachineParameters};
use crate::prelude::{Handle, NativeAccess, NativeDrop};
use std::ffi::{CStr, CString};
use std::fmt;
use std::mem::forget;
use std::os::raw::{c_char, c_void};

pub type InterpreterParameters = Handle<NativeVMParameters>;
unsafe impl Send for InterpreterParameters {}
unsafe impl Sync for InterpreterParameters {}

impl NativeDrop for NativeVMParameters {
    fn drop(&mut self) {
        unsafe {
            vm_parameters_destroy(self);
        }
    }
}

impl InterpreterParameters {
    pub fn from_args<P: AsRef<str>>(arguments: Vec<P>) -> Self {
        let vars = std::env::vars()
            .map(|arg| CString::new(format!("{}={}", arg.0, arg.1)).unwrap())
            .collect::<Vec<CString>>();

        let mut c_vars = vars
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut default_parameters = Self::default();
        default_parameters.set_arguments(arguments);
        default_parameters.native_mut().environmentVector = c_vars.as_mut_ptr();

        forget(vars);
        forget(c_vars);

        if default_parameters.has_arguments() {
            unsafe {
                vm_parameters_parse(
                    default_parameters.native().processArgc,
                    default_parameters.native().processArgv,
                    default_parameters.native_mut(),
                )
            };
        }

        default_parameters
    }

    pub fn from_env_args() -> Self {
        Self::from_args(std::env::args().collect())
    }

    pub fn image_file_name(&self) -> &str {
        if self.native().imageFileName.is_null() {
            return "";
        }

        let c_str: &CStr = unsafe { CStr::from_ptr(self.native().imageFileName) };
        let str_slice: &str = c_str.to_str().unwrap();
        str_slice
    }

    pub fn set_image_file_name(&mut self, file_name: impl Into<String>) {
        let new_image_name: String = file_name.into();

        if self.image_file_name() == new_image_name {
            return;
        }

        let previous_file_name = self.native().imageFileName as *mut c_void;
        unsafe { free(previous_file_name) };

        let c_str = CString::new(new_image_name).unwrap();
        self.native_mut().imageFileName = c_str.into_raw();
        self.native_mut().isDefaultImage = false;
    }

    pub fn arguments(&self) -> Vec<String> {
        let args_ptr = self.native().processArgv as *mut *mut c_char;
        let args_length = self.native().processArgc as usize;

        let arg_ptrs: Vec<*mut c_char> =
            unsafe { Vec::from_raw_parts(args_ptr, args_length, args_length) };
        let arguments: Vec<String> = arg_ptrs
            .iter()
            .map(|each| unsafe { CStr::from_ptr(*each).to_string_lossy().into_owned() })
            .collect();

        std::mem::forget(arg_ptrs);
        arguments
    }

    pub fn has_arguments(&self) -> bool {
        self.native().processArgc > 0
    }

    pub fn set_arguments<P: AsRef<str>>(&mut self, arguments: Vec<P>) {
        // create a vector of zero terminated strings
        let mut args = arguments
            .iter()
            .map(|each| each.as_ref().to_string())
            .map(|each| CString::into_raw(CString::new(each).unwrap()))
            .collect::<Vec<*mut c_char>>();

        args.shrink_to_fit();

        let args_ptr = args.as_ptr() as *mut *const c_char;
        let args_length = args.len() as i32;
        std::mem::forget(args);

        if !self.native().processArgv.is_null() {
            let previous_ptr = self.native().processArgv as *mut *mut c_char;
            let previous_length = self.native().processArgc as usize;

            let previous_arg_ptrs: Vec<*mut c_char> =
                unsafe { Vec::from_raw_parts(previous_ptr, previous_length, previous_length) };
            let previous_args = previous_arg_ptrs
                .iter()
                .map(|each| unsafe { CString::from_raw(*each) })
                .collect::<Vec<CString>>();

            drop(previous_args);
            drop(previous_arg_ptrs);
        }

        self.native_mut().processArgv = args_ptr;
        self.native_mut().processArgc = args_length;
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

    pub fn is_worker(&self) -> bool {
        self.native().isWorker
    }

    pub fn set_is_worker(&mut self, is_worker: bool) {
        self.native_mut().isWorker = is_worker;
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

    pub fn image_parameters(&self) -> &ImageParameters {
        ImageParameters::borrow_from_native(&self.native().imageParameters)
    }

    pub fn virtual_machine_parameters(&self) -> &VirtualMachineParameters {
        VirtualMachineParameters::borrow_from_native(&self.native().vmParameters)
    }
}

impl fmt::Debug for InterpreterParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VMParameters")
            .field("image_file_name", &self.image_file_name())
            .field("arguments", &self.arguments())
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
            .field("image_parameters", &self.image_parameters().as_vec())
            .field(
                "virtual_machine_parameters",
                &self.virtual_machine_parameters().as_vec(),
            )
            .finish()
    }
}

impl Default for InterpreterParameters {
    fn default() -> Self {
        Self::from_native_c(NativeVMParameters {
            imageFileName: std::ptr::null_mut(),
            isDefaultImage: false,
            defaultImageFound: false,
            isInteractiveSession: false,
            isWorker: false,
            maxStackFramesToPrint: 0,
            maxOldSpaceSize: 0,
            maxCodeSize: 0,
            edenSize: 0,
            minPermSpaceSize: 0,
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
