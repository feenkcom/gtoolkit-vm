use libffi::low::{ffi_cif, ffi_type, CodePtr};
use std::mem::{size_of, transmute};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use vm_bindings::{Marshallable, ObjectFieldIndex, StackOffset};

use crate::{vm, EventLoopCallout, EventLoopMessage};

#[repr(u16)]
enum TFExternalFunction {
    Handle,
    Definition,
    FunctionName,
    ModuleName,
}

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

    let external_function_oop = proxy.stack_object_value(StackOffset::new(
        TFPrimitiveCallout::ExternalFunction as i32,
    ));
    let external_function = proxy.get_handler(external_function_oop);

    let cif_oop = proxy.object_field_at(
        external_function_oop,
        ObjectFieldIndex::new(TFExternalFunction::Definition as usize),
    );

    let cif_ptr = proxy.get_handler(cif_oop) as *mut ffi_cif;
    let cif: &ffi_cif = unsafe { transmute(cif_ptr) };

    let semaphore_index = proxy
        .stack_integer_value(StackOffset::new(TFPrimitiveCallout::SemaphoreIndex as i32))
        as usize;

    let arguments_array_oop =
        proxy.stack_object_value(StackOffset::new(TFPrimitiveCallout::Arguments as i32));
    let argument_size: usize = cif.nargs as usize;

    let arg_types: &[*mut ffi_type] =
        unsafe { std::slice::from_raw_parts_mut(cif.arg_types, argument_size as usize) };

    let parameters = if argument_size > 0 {
        Some(proxy.calloc(argument_size, size_of::<*mut c_void>()) as *mut *mut c_void)
    } else {
        None
    };

    if parameters.is_some() {
        let mut parameters_slice =
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
    }));

    vm().send(EventLoopMessage::Call(callout.clone())).unwrap();

    // intentionally leak the callout so that it can be released later, once the return value is read
    let callout_ptr = Arc::into_raw(callout);

    proxy.method_return_value(proxy.new_external_address(callout_ptr));
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

        let callout_address_oop = proxy.stack_object_value(StackOffset::new(
            TFPrimitiveReturnValue::CalloutAddress as i32,
        ));
        let callout_address =
            proxy.read_address(callout_address_oop) as *mut Mutex<EventLoopCallout>;

        if callout_address.is_null() {
            return proxy.primitive_fail();
        }

        let mut callout = Arc::from_raw(callout_address);

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
