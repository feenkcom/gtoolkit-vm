use crate::bindings::{getHandler, isVMRunOnWorkerThread, sqInt, VirtualMachine, instantiateClassindexableSize, BytesPerWord, usqInt, firstIndexableField, readAddress};

use std::sync::mpsc::Sender;

use crate::cointerp::{marshallArgumentFromatIndexintoofTypewithSize, numSlotsOf, instantiateClassindexableSizeisPinned, marshallAndPushReturnValueFromofTypepoping};
use crate::prelude::NativeTransmutable;
use libc::c_char;
use libc::c_int;
use libffi::low::{ffi_cif, ffi_type, CodePtr};


use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::intrinsics::transmute;

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

    pub fn get_handler(&self, object: ObjectPointer) -> *mut c_void {
        unsafe { getHandler(object.into_native()) }
    }

    pub fn read_address(&self, external_address_object: ObjectPointer) -> *mut c_void {
        unsafe { readAddress(external_address_object.into_native()) }
    }

    pub fn stack_integer_value(&self, offset: StackOffset) -> sqInt {
        let function = self.interpreter.stackIntegerValue.unwrap();
        unsafe { function(offset.into_native()) }
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

    pub fn checked_integer_value_of(&self, object: ObjectPointer) -> sqInt {
        let function = self.interpreter.checkedIntegerValueOf.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn marshall_argument_from_at_index_into_of_type_with_size(
        &self,
        arguments: ObjectPointer,
        index: usize,
        arg_holder: sqInt,
        arg_type: sqInt,
        arg_type_size: sqInt,
    ) {
        unsafe {
            marshallArgumentFromatIndexintoofTypewithSize(
                arguments.into_native(),
                index as sqInt,
                arg_holder,
                arg_type,
                arg_type_size,
            )
        };
    }

    pub fn marshall_and_push_return_value_of_type_popping(
        &self,
        return_holder: *mut c_void,
        return_type: &ffi_type,
        primitive_arguments_and_receiver_count: usize,
    ) {
        unsafe {
            let return_type_ptr: *mut ffi_type = transmute(return_type);

            marshallAndPushReturnValueFromofTypepoping(
                return_holder as sqInt,
                return_type_ptr,
                primitive_arguments_and_receiver_count as sqInt
            )
        };
    }

    pub fn instantiate_indexable_class_of_size(&self, class: ObjectPointer, size: usize, is_pinned: bool) -> ObjectPointer {
        let oop = unsafe { instantiateClassindexableSizeisPinned(
            class.into_native(),
            size as usqInt,
            is_pinned as sqInt
        ) };

        ObjectPointer::from_native_c(oop)
    }

    pub fn new_external_address(&self) -> ObjectPointer {
        let external_address_class = self.class_external_address();
        self.instantiate_indexable_class_of_size(external_address_class, std::mem::size_of::<c_void>(), true)
    }

    pub fn new_external_address_from_pointer<T>(&self, ptr: *const T) -> ObjectPointer {
        let external_address = self.new_external_address();

        let external_address_ptr = unsafe { firstIndexableField(external_address.into_native()) as *mut usize };
        unsafe { *external_address_ptr = ptr as usize };
        external_address
    }

    pub fn pop_then_push(&self, amount_of_stack_items: usize, object: ObjectPointer) {
        let function = self.interpreter.popthenPush.unwrap();
        unsafe { function(amount_of_stack_items as sqInt, object.into_native()); }
    }

    pub fn is_on_worker_thread(&self) -> bool {
        unsafe { isVMRunOnWorkerThread() != 0 }
    }

    pub fn send(&self, request: GToolkitVMRequest) {
        self.sender.send(request);
    }

    pub fn call(&self, callout: GToolkitCallout) {
        println!("Requesting callout {:?}", &callout);
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
    pub(crate) args: Option<*mut *mut c_void>,
    pub(crate) result: Option<*mut c_void>,
    pub(crate) semaphore: sqInt,
}

impl GToolkitCallout {
    pub fn call(&self) {
        unsafe {
            libffi::raw::ffi_call(
                self.cif,
                Some(*unsafe { self.func.as_safe_fun() }),
                self.result.unwrap_or(std::ptr::null_mut()),
                self.args.unwrap_or(std::ptr::null_mut()),
            )
        }
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

pub enum GToolkitVMRequest {
    Terminate,
    Call(GToolkitCallout),
    WakeUp,
}
