use crate::bindings::{
    calloc, exportGetHandler as getHandler,
    exportInstantiateClassIsPinned as instantiateClassIsPinned, exportReadAddress as readAddress,
    free, malloc, memcpy, sqInt, usqInt, VirtualMachine as sqInterpreterProxy,
};

use std::ffi::CString;
use std::mem::size_of;

use crate::prelude::{Handle, NativeAccess, NativeDrop, NativeTransmutable};
use std::os::raw::{c_char, c_double, c_void};

pub type InterpreterProxy = Handle<sqInterpreterProxy>;
impl NativeDrop for sqInterpreterProxy {
    fn drop(&mut self) {}
}

impl InterpreterProxy {
    pub fn major_version(&self) -> usize {
        let function = self.native().majorVersion.unwrap();
        unsafe { function() as usize }
    }

    pub fn minor_version(&self) -> usize {
        let function = self.native().minorVersion.unwrap();
        unsafe { function() as usize }
    }

    pub fn get_stack_pointer(&self) -> *mut sqInt {
        let function = self.native().getStackPointer.unwrap();
        unsafe { function() }
    }

    pub fn true_object(&self) -> ObjectPointer {
        let function = self.native().trueObject.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn false_object(&self) -> ObjectPointer {
        let function = self.native().falseObject.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn class_array(&self) -> ObjectPointer {
        let function = self.native().classArray.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn class_external_address(&self) -> ObjectPointer {
        let function = self.native().classExternalAddress.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn class_string(&self) -> ObjectPointer {
        let function = self.native().classString.unwrap();
        unsafe { ObjectPointer::from_native_c(function()) }
    }

    pub fn stack_object_value(&self, offset: StackOffset) -> ObjectPointer {
        let function = self.native().stackObjectValue.unwrap();
        unsafe { ObjectPointer::from_native_c(function(offset.into_native())) }
    }

    pub fn stack_integer_value(&self, offset: StackOffset) -> sqInt {
        let function = self.native().stackIntegerValue.unwrap();
        unsafe { function(offset.into_native()) }
    }

    pub fn pop(&self, amount_of_stack_items: usize) {
        let function = self.native().pop.unwrap();
        unsafe {
            function(amount_of_stack_items as sqInt);
        }
    }

    pub fn pop_then_push(&self, amount_of_stack_items: usize, object: ObjectPointer) {
        let function = self.native().popthenPush.unwrap();
        unsafe {
            function(amount_of_stack_items as sqInt, object.into_native());
        }
    }

    pub fn pop_then_push_integer(&self, amount_of_stack_items: usize, number: impl Into<sqInt>) {
        self.pop_then_push(amount_of_stack_items, self.new_integer(number));
    }

    pub fn get_handler(&self, object: ObjectPointer) -> *mut c_void {
        unsafe { getHandler(object.into_native()) }
    }

    /// Return an item at an index within the indexable object (array, string, etc.).
    /// The index must start from 1, and not 0 like in Rust
    pub fn item_at(
        &self,
        indexable_object: ObjectPointer,
        field_index: ObjectFieldIndex,
    ) -> ObjectPointer {
        let function = self.native().stObjectat.unwrap();
        unsafe {
            ObjectPointer::from_native_c(function(
                indexable_object.into_native(),
                field_index.into_native(),
            ))
        }
    }

    pub fn item_at_put(
        &self,
        indexable_object: ObjectPointer,
        field_index: ObjectFieldIndex,
        value_object: ObjectPointer,
    ) -> ObjectPointer {
        let function = self.native().stObjectatput.unwrap();
        unsafe {
            ObjectPointer::from_native_c(function(
                indexable_object.into_native(),
                field_index.into_native(),
                value_object.into_native(),
            ))
        }
    }

    pub fn object_field_at(
        &self,
        object: ObjectPointer,
        field_index: ObjectFieldIndex,
    ) -> ObjectPointer {
        let function = self.native().fetchPointerofObject.unwrap();
        // watch out! the interpreter function expects to get field index first and object second
        unsafe {
            ObjectPointer::from_native_c(function(field_index.into_native(), object.into_native()))
        }
    }

    pub fn integer_value_of(&self, object: ObjectPointer) -> sqInt {
        let function = self.native().integerValueOf.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn checked_integer_value_of(&self, object: ObjectPointer) -> sqInt {
        let function = self.native().checkedIntegerValueOf.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn cstring_value_of(&self, object: ObjectPointer) -> Option<CString> {
        let function = self.native().cStringOrNullFor.unwrap();
        let buffer: *mut c_char = unsafe { function(object.into_native()) };
        if buffer.is_null() {
            None
        } else {
            unsafe { Some(CString::from_raw(buffer)) }
        }
    }

    pub fn is_character_object(&self, object: ObjectPointer) -> bool {
        let function = self.native().isCharacterObject.unwrap();
        unsafe { function(object.into_native()) != 0 }
    }

    pub fn character_value_of(&self, object: ObjectPointer) -> c_char {
        let function = self.native().characterValueOf.unwrap();
        unsafe { function(object.into_native()) as c_char }
    }

    pub fn positive_32bit_value_of(&self, object: ObjectPointer) -> u32 {
        let function = self.native().positive32BitValueOf.unwrap();
        unsafe { function(object.into_native()) as u32 }
    }

    pub fn positive_64bit_value_of(&self, object: ObjectPointer) -> u64 {
        let function = self.native().positive64BitValueOf.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn fetch_float_at(&self, object: ObjectPointer, index: ObjectFieldIndex) -> c_double {
        let function = self.native().fetchFloatofObject.unwrap();
        unsafe { function(index.into_native(), object.into_native()) }
    }

    pub fn instantiate_indexable_class_of_size(
        &self,
        class: ObjectPointer,
        size: usize,
    ) -> ObjectPointer {
        let function = self.native().instantiateClassindexableSize.unwrap();
        let oop = unsafe { function(class.into_native(), size as sqInt) };

        ObjectPointer::from_native_c(oop)
    }

    pub fn signal_semaphore(&self, index: usize) {
        let function = self.native().signalSemaphoreWithIndex.unwrap();
        unsafe {
            function(index as sqInt);
        }
    }

    pub fn method_return_value(&self, value: ObjectPointer) {
        let function = self.native().methodReturnValue.unwrap();
        unsafe { function(value.into_native()) };
    }

    pub fn method_return_boolean(&self, value: bool) {
        let function = self.native().methodReturnBool.unwrap();
        let boolean = if value {
            self.true_object()
        } else {
            self.false_object()
        };
        unsafe { function(boolean.into_native()) };
    }

    pub fn first_indexable_field(&self, object: ObjectPointer) -> *mut c_void {
        let function = self.native().firstIndexableField.unwrap();
        unsafe { function(object.into_native()) }
    }

    pub fn is_kind_of_class(&self, object: ObjectPointer, class: ObjectPointer) -> bool {
        let function = self.native().isKindOfClass.unwrap();
        unsafe { function(object.into_native(), class.into_native()) != 0 }
    }

    pub fn new_integer(&self, number: impl Into<sqInt>) -> ObjectPointer {
        let function = self.native().integerObjectOf.unwrap();
        let oop = unsafe { function(number.into()) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn new_string(&self, string: impl AsRef<str>) -> ObjectPointer {
        let function = self.native().stringForCString.unwrap();
        let rust_str = string.as_ref();
        let c_string = CString::new(rust_str).unwrap();

        let oop = unsafe { function(c_string.as_ptr() as *mut c_char) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn new_external_address<T>(&self, address: *const T) -> ObjectPointer {
        let external_address = self.instantiate_indexable_class_of_size(
            self.class_external_address(),
            size_of::<*mut c_void>(),
        );
        unsafe {
            *(self.first_indexable_field(external_address) as *mut *mut c_void) =
                address as *mut c_void
        };
        external_address
    }

    pub fn new_positive_64bit_integer(&self, integer: u64) -> ObjectPointer {
        let function = self.native().positive64BitIntegerFor.unwrap();
        let oop = unsafe { function(integer) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn read_address(&self, external_address_object: ObjectPointer) -> *mut c_void {
        unsafe { readAddress(external_address_object.into_native()) }
    }

    pub fn malloc(&self, bytes: usize) -> *mut c_void {
        unsafe { malloc(bytes.try_into().unwrap()) }
    }

    pub fn calloc(&self, amount: usize, size: usize) -> *mut c_void {
        unsafe { calloc(amount.try_into().unwrap(), size.try_into().unwrap()) }
    }

    pub fn free(&self, address: *mut c_void) {
        unsafe {
            free(address);
        }
    }

    pub fn primitive_fail(&self) {
        let function = self.native().primitiveFail.unwrap();
        unsafe { function() };
    }

    pub fn is_failed(&self) -> bool {
        let function = self.native().failed.unwrap();
        unsafe { function() != 0 }
    }
}

pub(crate) fn write_value<T>(value: T, holder: *mut c_void) {
    let holder = holder as *mut T;
    unsafe { *holder = value };
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectPointer(sqInt);
impl NativeTransmutable<sqInt> for ObjectPointer {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectFieldIndex(sqInt);
impl NativeTransmutable<sqInt> for ObjectFieldIndex {}
impl ObjectFieldIndex {
    pub fn new(index: usize) -> Self {
        Self(index as sqInt)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct StackOffset(sqInt);
impl StackOffset {
    pub fn new(offset: i32) -> Self {
        Self(offset as sqInt)
    }
}
impl NativeTransmutable<sqInt> for StackOffset {}
