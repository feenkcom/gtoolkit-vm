use crate::bindings::{isVMRunOnWorkerThread, sqInt, VirtualMachine};

use std::sync::mpsc::Sender;

use crate::cointerp::numSlotsOf;
use crate::prelude::NativeTransmutable;
use libc::c_char;
use libc::c_int;
use libffi::low::{ffi_cif, CodePtr};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct GToolkitVM {
    sender: Sender<GToolkitVMRequest>,
    interpreter: VirtualMachine,
}

pub trait GToolkitVMPointer {
    fn with<DefaultBlock, Block, Return>(&self, default: DefaultBlock, block: Block) -> Return
    where
        DefaultBlock: FnOnce() -> Return,
        Block: FnOnce(&GToolkitVM) -> Return;

    fn with_not_null<Block>(&self, block: Block)
    where
        Block: FnOnce(&GToolkitVM);
}

impl GToolkitVMPointer for *const Mutex<Option<GToolkitVM>> {
    fn with<DefaultBlock, Block, Return>(&self, default: DefaultBlock, block: Block) -> Return
    where
        DefaultBlock: FnOnce() -> Return,
        Block: FnOnce(&GToolkitVM) -> Return,
    {
        if self.is_null() {
            return default();
        }

        let gt_vm_arc = unsafe { Arc::from_raw(*self) };

        let result = block(
            gt_vm_arc
                .lock()
                .expect("Could not lock the mutex")
                .as_ref()
                .expect("Could not get GTVM"),
        );

        Arc::into_raw(gt_vm_arc);
        result
    }

    fn with_not_null<Block>(&self, block: Block)
    where
        Block: FnOnce(&GToolkitVM),
    {
        self.with(|| {}, block);
    }
}

unsafe impl Send for GToolkitVM {}
unsafe impl Sync for GToolkitVM {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectPointer(sqInt);
impl NativeTransmutable<sqInt> for ObjectPointer {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectFieldIndex(sqInt);
impl NativeTransmutable<sqInt> for ObjectFieldIndex {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct StackOffset(sqInt);
impl NativeTransmutable<sqInt> for StackOffset {}

impl GToolkitVM {
    pub fn new(sender: Sender<GToolkitVMRequest>, interpreter: VirtualMachine) -> Self {
        Self {
            sender,
            interpreter,
        }
    }

    pub fn major_version(&self) -> usize {
        let function = self.interpreter.majorVersion.unwrap();
        unsafe { function() as usize }
    }

    pub fn minor_version(&self) -> usize {
        let function = self.interpreter.minorVersion.unwrap();
        unsafe { function() as usize }
    }

    pub fn get_stack_pointer(&self) -> *mut sqInt {
        let function = self.interpreter.getStackPointer.unwrap();
        unsafe { function() }
    }

    pub fn true_object(&self) -> ObjectPointer {
        let function = self.interpreter.trueObject.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn false_object(&self) -> ObjectPointer {
        let function = self.interpreter.falseObject.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn class_external_address(&self) -> ObjectPointer {
        let function = self.interpreter.classExternalAddress.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn stack_object_value(&self, offset: StackOffset) -> ObjectPointer {
        let function = self.interpreter.stackObjectValue.unwrap();
        unsafe { ObjectPointer::from_native_c(function(offset.into_native())) }
    }

    pub fn array_item_at(
        &self,
        object: ObjectPointer,
        field_index: ObjectFieldIndex,
    ) -> ObjectPointer {
        let function = self.interpreter.stObjectat.unwrap();
        unsafe {
            ObjectPointer::from_native_c(function(object.into_native(), field_index.into_native()))
        }
    }

    pub fn object_field_at(
        &self,
        object: ObjectPointer,
        field_index: ObjectFieldIndex,
    ) -> ObjectPointer {
        let function = self.interpreter.fetchPointerofObject.unwrap();
        // watch out! the interpreter function expects to get field index first and object second
        unsafe {
            ObjectPointer::from_native_c(function(field_index.into_native(), object.into_native()))
        }
    }

    /// Return the amount of slots in an object
    pub fn object_num_slots(&self, object: ObjectPointer) -> usize {
        unsafe { numSlotsOf(object.into_native()) as usize }
    }

    pub fn integer_value_of(&self, object: ObjectPointer) -> sqInt {
        let function = self.interpreter.integerValueOf.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn is_on_worker_thread(&self) -> bool {
        unsafe { isVMRunOnWorkerThread() != 0 }
    }

    pub fn send(&self, request: GToolkitVMRequest) {
        self.sender.send(request);
    }

    pub fn call(&self, callout: GToolkitCallout) {
        self.send(GToolkitVMRequest::Call(callout));
    }

    pub fn wake_up(&self) {
        println!("Sending wake up");
        self.send(GToolkitVMRequest::WakeUp);
    }

    pub fn terminate(&self) {
        self.send(GToolkitVMRequest::Terminate)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GToolkitCallout {
    pub(crate) cif: *mut ffi_cif,
    pub(crate) func: CodePtr,
    pub(crate) args: *mut *mut c_void,
    pub(crate) result: *mut c_void,
    pub(crate) semaphore: sqInt,
}

impl GToolkitCallout {
    pub fn call(&self) {
        unsafe {
            libffi::raw::ffi_call(
                self.cif,
                Some(*unsafe { self.func.as_safe_fun() }),
                self.result,
                self.args,
            )
        }
    }
}

pub enum GToolkitVMRequest {
    Terminate,
    Call(GToolkitCallout),
    WakeUp,
}
