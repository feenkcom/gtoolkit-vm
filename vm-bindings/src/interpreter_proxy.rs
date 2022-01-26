use crate::bindings::{sqInt, VirtualMachine as sqInterpreterProxy};
use std::ffi::CString;
use std::mem::size_of;

use crate::prelude::{Handle, NativeAccess, NativeDrop, NativeTransmutable};
use std::os::raw::{c_char, c_void};

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

    pub fn pop_then_push(&self, amount_of_stack_items: usize, object: ObjectPointer) {
        let function = self.native().popthenPush.unwrap();
        unsafe {
            function(amount_of_stack_items as sqInt, object.into_native());
        }
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

    pub fn new_string(&self, string: impl AsRef<str>) -> ObjectPointer {
        let function = self.native().stringForCString.unwrap();
        let rust_str = string.as_ref();
        let c_string = CString::new(rust_str).unwrap();

        let oop = unsafe { function(c_string.as_ptr() as *mut c_char) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn new_external_address(&self, address: *const c_void) -> ObjectPointer {
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
impl NativeTransmutable<sqInt> for StackOffset {}
