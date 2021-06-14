#[macro_use]
extern crate lazy_static;

pub mod bindings;
pub mod cointerp;
pub mod prelude;
pub mod vm;

use std::ffi::CString;

use libc::c_char;
use libc::c_int;
use libffi::low::{ffi_cif, ffi_type, CodePtr};

use crate::bindings::{
    calloc, checkedLongAtput, firstIndexableField, free, instantiateClassindexableSize,
    isVMRunOnWorkerThread, loadModuleHandle, malloc, memcpy, methodArgumentCount,
    signalSemaphoreWithIndex, sqGetInterpreterProxy, sqInt, stackObjectValue, usqInt,
    vm_main_with_parameters, vm_parameters_parse, vm_run_interpreter, BytesPerWord,
    VMParameterVector, VirtualMachine, TRUE,
};

use crate::prelude::NativeTransmutable;
use crate::vm::{gtoolkit_null_semaphore_signaller, GToolkitCallout, GToolkitVM, GToolkitVMPointer, GToolkitVMRequest, ObjectFieldIndex, StackOffset, gtoolkit_null_receiver_signaller, gtoolkit_receiver_signaller};
use bindings::VMParameters;
use std::cell::RefCell;
use std::intrinsics::transmute;
use std::mem::size_of;
use std::os::raw::{c_uint, c_void};
use std::ptr::slice_from_raw_parts_mut;
use std::sync::mpsc::{channel, RecvError, Sender};
use std::sync::{Arc, Mutex};

unsafe impl Send for VMParameters {}

#[cfg(target_os = "android")]
ndk_glue::ndk_glue!(app_main);

lazy_static! {
    pub static ref GTVM: Arc<Mutex<Option<GToolkitVM>>> = Arc::new(Mutex::new(None));
}

/// Returns an ExternalAddress that points to the GTVM
#[no_mangle]
pub fn primitiveGetAddressOfGToolkitVM() {
    unsafe {
        let gtvm_raw = unsafe { Arc::into_raw(GTVM.clone()) };

        let gt_lock = GTVM.lock().unwrap();
        let gt = gt_lock.as_ref().unwrap();

        let mut stack_pointer = gt.get_stack_pointer();

        let external_address = gt.new_external_address_from_pointer(gtvm_raw);

        *stack_pointer = external_address.into_native();
    }
}

#[repr(u16)]
enum TFPrimitiveCallout {
    SemaphoreIndex,
    Arguments,
    ExternalFunction,
    Receiver,
}

#[repr(u16)]
enum TFExternalFunction {
    Handle,
    Definition,
    FunctionName,
    ModuleName,
}

#[no_mangle]
pub fn primitiveMainThreadCalloutGToolkitVM() {
    unsafe {
        let gt_lock = GTVM.lock().unwrap();
        let gt = gt_lock.as_ref().unwrap();

        let external_function_oop = gt.stack_object_value(StackOffset::from_native_c(
            TFPrimitiveCallout::ExternalFunction as sqInt,
        ));
        let external_function = gt.get_handler(external_function_oop);

        let cif_oop = gt.object_field_at(
            external_function_oop,
            ObjectFieldIndex::from_native_c(TFExternalFunction::Definition as sqInt),
        );

        let cif_ptr = gt.get_handler(cif_oop) as *mut ffi_cif;
        let cif: &ffi_cif = transmute(cif_ptr);

        let semaphore_index = gt.stack_integer_value(StackOffset::from_native_c(
            TFPrimitiveCallout::SemaphoreIndex as sqInt,
        ));

        let arguments_array_oop = gt.stack_object_value(StackOffset::from_native_c(
            TFPrimitiveCallout::Arguments as sqInt,
        ));
        let argument_size: usize = cif.nargs as usize;

        let arg_types: &[*mut ffi_type] =
            std::slice::from_raw_parts_mut(cif.arg_types, argument_size as usize);

        let parameters = if argument_size > 0 {
            Some(calloc(argument_size as usqInt, size_of::<c_void>() as usqInt) as *mut *mut c_void)
        } else {
            None
        };

        if parameters.is_some() {
            let mut parameters_slice =
                std::slice::from_raw_parts_mut(parameters.unwrap(), argument_size);

            for argument_index in 0..argument_size {
                let arg_type: &mut ffi_type = transmute(arg_types[argument_index]);

                let arg_holder = malloc(arg_type.size as u64);
                parameters_slice[argument_index] = arg_holder;

                gt.marshall_argument_from_at_index_into_of_type_with_size(
                    arguments_array_oop,
                    argument_index,
                    arg_holder as sqInt,
                    arg_type.type_ as sqInt,
                    arg_type.size as sqInt,
                );
            }
        }

        let return_type: &ffi_type = transmute(cif.rtype);
        let return_holder = if return_type.size > 0 {
            Some(malloc(return_type.size as usqInt))
        } else {
            None
        };

        let callout = GToolkitCallout {
            cif: cif_ptr,
            func: CodePtr(external_function),
            args: parameters,
            result: return_holder,
            semaphore: semaphore_index,
        };

        gt.call(callout);

        // intentionally leak the callout so that it can be released later, once the return value is read
        let callout_ptr = Box::into_raw(Box::new(callout));

        gt.pop_then_push(4, gt.new_external_address_from_pointer(callout_ptr));
    }
}

#[repr(u16)]
enum TFPrimitiveReturnValue {
    CalloutAddress,
    Receiver,
}
#[no_mangle]
pub fn primitiveExtractReturnValueGToolkitVM() {
    unsafe {
        let gt_lock = GTVM.lock().unwrap();
        let gt = gt_lock.as_ref().unwrap();

        let callout_address_oop = gt.stack_object_value(StackOffset::from_native_c(
            TFPrimitiveReturnValue::CalloutAddress as sqInt,
        ));
        let callout_address = gt.read_address(callout_address_oop) as *mut GToolkitCallout;

        let mut callout = Box::from_raw(callout_address);

        if callout.result.is_some() {
            let return_holder = callout.result.unwrap();

            gt.marshall_and_push_return_value_of_type_popping(
                return_holder,
                callout.return_type(),
                2, // one for the argument + one for the receiver
            );
        }

        // start freeing memory:
        // - arguments
        // - return holder
        if callout.args.is_some() {
            let arguments = callout.args.unwrap();
            let arguments_size: usize = callout.number_of_arguments();

            let mut arguments_slice = std::slice::from_raw_parts_mut(arguments, arguments_size);
            for index in 0..arguments_size {
                let argument = arguments_slice[index];
                if !argument.is_null() {
                    free(argument);
                }
            }

            free(arguments as *mut c_void);
            callout.args = None;
        }

        if callout.result.is_some() {
            let return_holder = callout.result.unwrap();
            if !return_holder.is_null() {
                free(return_holder);
            }
            callout.result = None;
        }

        drop(callout);
    }
}

#[no_mangle]
pub fn gtoolkit_vm_major_version(gt_vm_ptr: *const Mutex<Option<GToolkitVM>>) -> usize {
    gt_vm_ptr.with(|| 0, |gt_vm| gt_vm.major_version())
}

#[no_mangle]
pub fn gtoolkit_vm_minor_version(gt_vm_ptr: *const Mutex<Option<GToolkitVM>>) -> usize {
    gt_vm_ptr.with(|| 0, |gt_vm| gt_vm.minor_version())
}

#[no_mangle]
pub fn gtoolkit_vm_wake_up(gt_vm_ptr: *const Mutex<Option<GToolkitVM>>) {
    gt_vm_ptr.with_not_null(|gt_vm| gt_vm.wake_up());
}

#[no_mangle]
pub fn gtoolkit_vm_is_on_worker_thread(gt_vm_ptr: *const Mutex<Option<GToolkitVM>>) -> bool {
    gt_vm_ptr.with(|| false, |gt_vm| gt_vm.is_on_worker_thread())
}

#[no_mangle]
pub fn gtoolkit_vm_get_semaphore_signaller(
    gt_vm_ptr: *const Mutex<Option<GToolkitVM>>,
    thunk_ret_ptr: &mut *const c_void
) -> unsafe extern "C" fn(usize, *const c_void) {
    *thunk_ret_ptr = gt_vm_ptr as *const c_void;
    gt_vm_ptr.with(
        || gtoolkit_null_semaphore_signaller as unsafe extern "C" fn(usize, *const c_void),
        |gt_vm| gt_vm.get_semaphore_signaller(),
    )
}

#[no_mangle]
pub fn gtoolkit_vm_get_receiver_signaller(
    gt_vm_ptr: *const Mutex<Option<GToolkitVM>>,
    thunk_ret_ptr: &mut *const c_void
) -> unsafe extern "C" fn(*const c_void) {
    *thunk_ret_ptr = gt_vm_ptr as *const c_void;
    gt_vm_ptr.with(
        || gtoolkit_null_receiver_signaller as unsafe extern "C" fn(*const c_void),
        |gt_vm| gtoolkit_receiver_signaller as unsafe extern "C" fn(*const c_void)
    )
}

#[no_mangle]
pub fn gtoolkit_vm_main_thread_callout(
    gt_vm_ptr: *const Mutex<Option<GToolkitVM>>,
    cif: *mut ffi_cif,
    func: *mut c_void,
    args: *mut c_void,
    result: *mut c_void,
    semaphore: sqInt,
) {
    gt_vm_ptr.with_not_null(|gt_vm| {
        gt_vm.call(GToolkitCallout {
            cif,
            func: CodePtr(func),
            args: Some(args as *mut *mut c_void),
            result: Some(result),
            semaphore,
        })
    });
}

pub fn app_main() {
    let (sender, receiver) = channel();



    unsafe {
        let interpreter: VirtualMachine = unsafe { *sqGetInterpreterProxy() };

        let mut gt_vm = GTVM.lock().unwrap();
        *gt_vm = Some(GToolkitVM::new(sender, transmute(&receiver), interpreter));
    };

    // create a vector of zero terminated strings
    let mut args = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect::<Vec<CString>>();

    // convert the strings to raw pointers
    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const c_char>>();

    let mut p = VMParameters {
        imageFileName: std::ptr::null_mut(),
        isDefaultImage: false,
        defaultImageFound: false,
        isInteractiveSession: false,
        maxStackFramesToPrint: 100,
        processArgc: 0,
        processArgv: std::ptr::null_mut(),
        environmentVector: std::ptr::null_mut(),
        vmParameters: VMParameterVector {
            count: 0,
            parameters: std::ptr::null_mut(),
        },
        imageParameters: VMParameterVector {
            count: 0,
            parameters: std::ptr::null_mut(),
        },
    };

    unsafe {
        vm_parameters_parse(c_args.len() as c_int, c_args.as_mut_ptr(), &mut p);
        vm_main_with_parameters(&mut p);
    }

    loop {
        GToolkitVM::process_request(receiver.recv());
    }
}