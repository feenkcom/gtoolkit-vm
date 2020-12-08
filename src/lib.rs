#[macro_use]
extern crate lazy_static;

pub mod bindings;
pub mod cointerp;
pub mod prelude;
pub mod vm;

use std::ffi::CString;

use libc::c_char;
use libc::c_int;
use libffi::low::{ffi_cif, CodePtr};

use crate::bindings::{
    checkedLongAtput, firstIndexableField, instantiateClassindexableSize, isVMRunOnWorkerThread,
    loadModuleHandle, memcpy, methodArgumentCount, signalSemaphoreWithIndex, sqGetInterpreterProxy,
    sqInt, stackObjectValue, vm_main_with_parameters, vm_parameters_parse, vm_run_interpreter,
    BytesPerWord, VMParameterVector, VirtualMachine, TRUE,
};

use crate::prelude::NativeTransmutable;
use crate::vm::{
    GToolkitCallout, GToolkitVM, GToolkitVMPointer, GToolkitVMRequest, ObjectFieldIndex,
    StackOffset,
};
use bindings::VMParameters;
use std::cell::RefCell;
use std::intrinsics::transmute;
use std::os::raw::c_void;
use std::ptr::slice_from_raw_parts;
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
        let gtvm_ptr = gtvm_raw as usize;

        let gt_lock = GTVM.lock().unwrap();
        let gt = gt_lock.as_ref().unwrap();

        let mut stack_pointer = gt.get_stack_pointer();
        let external_address_class = gt.class_external_address();
        let external_address = instantiateClassindexableSize(
            external_address_class.into_native(),
            std::mem::size_of_val(&gtvm_ptr) as sqInt,
        );

        let external_address_ptr = firstIndexableField(external_address) as *mut usize;
        *external_address_ptr = gtvm_ptr;

        *stack_pointer = external_address as sqInt;
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
            args: args as *mut *mut c_void,
            result,
            semaphore,
        })
    });
}

pub fn app_main() {
    let (sender, receiver) = channel();

    unsafe {
        let interpreter: VirtualMachine = unsafe { *sqGetInterpreterProxy() };

        let mut gt_vm = GTVM.lock().unwrap();
        *gt_vm = Some(GToolkitVM::new(sender, interpreter));
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
        vm_parameters_parse(c_args.len() as c_int, c_args.as_mut_ptr(), (&mut p));
        vm_main_with_parameters(&mut p);
    }

    loop {
        match receiver.recv() {
            Ok(GToolkitVMRequest::Call(callout)) => {
                println!("GT was told to call {:?}", callout);
                callout.call();
                unsafe {
                    signalSemaphoreWithIndex(callout.semaphore);
                }
            }
            Ok(GToolkitVMRequest::Terminate) => {
                println!("GT was told to terminate");
                break;
            }
            Err(error) => {
                println!("[Error] {:?}", error);
                break;
            }
            Ok(GToolkitVMRequest::WakeUp) => {
                println!("GT woke up");
            }
        }
    }
}
