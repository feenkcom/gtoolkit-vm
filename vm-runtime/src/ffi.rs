use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::mem::{size_of, transmute};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

use libffi::low::{ffi_cif, ffi_type, CodePtr};

use vm_bindings::{Marshallable, ObjectFieldIndex, Smalltalk, StackOffset};

use crate::{vm, EventLoopMessage};

#[cfg(not(feature = "ffi"))]
compile_error!("\"ffi\" feature must be enabled for this module.");

#[repr(C)]
pub struct EventLoopCallout {
    pub(crate) function_name: Option<CString>,
    pub(crate) module_name: Option<CString>,
    pub(crate) cif: *mut ffi_cif,
    pub(crate) func: CodePtr,
    pub(crate) args: Option<*mut *mut c_void>,
    pub(crate) result: Option<*mut c_void>,
    pub(crate) callback: Option<Box<dyn FnOnce()>>,
}

impl EventLoopCallout {
    pub fn call(&mut self) {
        unsafe {
            libffi::raw::ffi_call(
                self.cif,
                Some(*(self.func.as_safe_fun())),
                self.result.unwrap_or(std::ptr::null_mut()),
                self.args.unwrap_or(std::ptr::null_mut()),
            )
        }
        let callback = self.callback.take().unwrap();
        callback();
    }

    pub fn return_type(&self) -> &ffi_type {
        let cif: &ffi_cif = unsafe { transmute(self.cif) };
        let rtype: &ffi_type = unsafe { transmute(cif.rtype) };
        rtype
    }

    pub fn number_of_arguments(&self) -> usize {
        let cif: &ffi_cif = unsafe { transmute(self.cif) };
        cif.nargs as usize
    }
}

impl Debug for EventLoopCallout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callout")
            .field("function_name", &self.function_name)
            .field("module_name", &self.module_name)
            .field("cif", &self.cif)
            .field("func", &self.func)
            .field("args", &self.args)
            .field("result", &self.result)
            .finish()
    }
}

#[allow(dead_code)]
#[repr(u16)]
enum TFExternalFunction {
    Handle,
    Definition,
    FunctionName,
    ModuleName,
}

#[allow(dead_code)]
#[repr(u16)]
enum TFPrimitiveCallout {
    SemaphoreIndex,
    Arguments,
    ExternalFunction,
    Receiver,
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveEventLoopCallout() {
    let proxy = vm().proxy();

    let external_function_oop = Smalltalk::stack_object_value(StackOffset::new(
        TFPrimitiveCallout::ExternalFunction as i32,
    ));
    let external_function = proxy.get_handler(external_function_oop);

    let cif_oop = Smalltalk::object_field_at(
        external_function_oop,
        ObjectFieldIndex::new(TFExternalFunction::Definition as usize),
    );

    let cif_ptr = proxy.get_handler(cif_oop) as *mut ffi_cif;
    let cif: &ffi_cif = unsafe { transmute(cif_ptr) };

    let function_name_oop = Smalltalk::object_field_at(
        external_function_oop,
        ObjectFieldIndex::new(TFExternalFunction::FunctionName as usize),
    );

    let function_name = if proxy.is_kind_of_class(function_name_oop, proxy.class_string()) {
        proxy.cstring_value_of(function_name_oop)
    } else {
        None
    };

    let module_name_oop = Smalltalk::object_field_at(
        external_function_oop,
        ObjectFieldIndex::new(TFExternalFunction::ModuleName as usize),
    );

    let module_name = if proxy.is_kind_of_class(module_name_oop, proxy.class_string()) {
        proxy.cstring_value_of(module_name_oop)
    } else {
        None
    };

    let semaphore_index =
        Smalltalk::stack_integer_value(StackOffset::new(TFPrimitiveCallout::SemaphoreIndex as i32))
            as usize;

    let arguments_array_oop =
        Smalltalk::stack_object_value(StackOffset::new(TFPrimitiveCallout::Arguments as i32));
    let argument_size: usize = cif.nargs as usize;

    let arg_types: &[*mut ffi_type] =
        unsafe { std::slice::from_raw_parts_mut(cif.arg_types, argument_size as usize) };

    let parameters = if argument_size > 0 {
        Some(proxy.calloc(argument_size, size_of::<*mut c_void>()) as *mut *mut c_void)
    } else {
        None
    };

    if parameters.is_some() {
        let parameters_slice =
            unsafe { std::slice::from_raw_parts_mut(parameters.unwrap(), argument_size) };

        for argument_index in 0..argument_size {
            let arg_type: &mut ffi_type = unsafe { transmute(arg_types[argument_index]) };
            parameters_slice[argument_index] = proxy
                .marshall_argument_from_at_index_into_of_type_with_size(
                    arguments_array_oop,
                    argument_index,
                    arg_type,
                )
                .unwrap();
        }
    }

    let return_type: &ffi_type = unsafe { transmute(cif.rtype) };
    let return_holder = if return_type.size > 0 {
        Some(proxy.malloc(return_type.size))
    } else {
        None
    };

    let callout = Arc::new(Mutex::new(EventLoopCallout {
        cif: cif_ptr,
        func: CodePtr(external_function),
        args: parameters,
        result: return_holder,
        callback: Some(Box::new(move || {
            vm().proxy().signal_semaphore(semaphore_index);
        })),
        function_name,
        module_name,
    }));

    vm().send(EventLoopMessage::Call(callout.clone())).unwrap();

    // if semaphore index is zero it means that nothing is waiting for the callout and we can just return nil.
    if semaphore_index == 0 {
        let callout_ptr: *const Mutex<EventLoopCallout> = std::ptr::null();
        Smalltalk::method_return_value(proxy.new_external_address(callout_ptr))
    } else {
        // intentionally leak the callout so that it can be released later, once the return value is read
        let callout_ptr = Arc::into_raw(callout);

        Smalltalk::method_return_value(proxy.new_external_address(callout_ptr));
    }
}

#[repr(u16)]
enum TFPrimitiveReturnValue {
    CalloutAddress,
    Receiver,
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveExtractReturnValue() {
    unsafe {
        let proxy = vm().proxy();

        let callout_address_oop = Smalltalk::stack_object_value(StackOffset::new(
            TFPrimitiveReturnValue::CalloutAddress as i32,
        ));
        let callout_address =
            proxy.read_address(callout_address_oop) as *mut Mutex<EventLoopCallout>;

        if callout_address.is_null() {
            return Smalltalk::primitive_fail();
        }

        let callout = Arc::from_raw(callout_address);

        let mut locked_callout = callout.lock().unwrap();

        if let Some(return_holder) = locked_callout.result {
            proxy
                .marshall_and_push_return_value_of_type_popping(
                    return_holder,
                    locked_callout.return_type(),
                    2, // one for the argument + one for the receiver
                )
                .expect("Failed to marshall the return value");
        } else {
            proxy.pop(1);
        }

        // start freeing memory:
        // - arguments
        // - return holder
        if let Some(arguments) = locked_callout.args {
            let arguments_size = locked_callout.number_of_arguments();

            let arguments_slice = std::slice::from_raw_parts_mut(arguments, arguments_size);
            for index in 0..arguments_size {
                let argument = arguments_slice[index];
                if !argument.is_null() {
                    proxy.free(argument);
                }
            }

            proxy.free(arguments as *mut c_void);
            locked_callout.args = None;
        }

        if let Some(return_holder) = locked_callout.result {
            if !return_holder.is_null() {
                proxy.free(return_holder);
            }
            locked_callout.result = None;
        }

        drop(locked_callout);
        drop(callout);
    }
}
