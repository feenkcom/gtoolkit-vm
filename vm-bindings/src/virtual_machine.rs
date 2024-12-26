use crate::{ObjectFieldIndex, ObjectPointer, StackOffset};
use std::os::raw::c_void;

use crate::bindings::{addressCouldBeClassObj, createNewMethodheaderbytecodeCount, ensureBehaviorHash, falseObject, fetchPointerofObject, firstBytePointerOfDataObject, firstIndexableField, hashBitsOf, instantiateClassindexableSize, instantiateClassisPinned, integerObjectOf, isOopForwarded, methodArgumentCount, methodReturnBool, methodReturnInteger, methodReturnValue, nilObject, primitiveFail, primitiveFailFor, sqInt, stObjectat, stObjectatput, stSizeOf, stackIntegerValue, stackObjectValue, trueObject};
use crate::prelude::NativeTransmutable;

pub struct Smalltalk {}

impl Smalltalk {
    pub fn method_argument_count() -> usize {
        unsafe { methodArgumentCount() as usize }
    }

    pub fn stack_object_value(offset: StackOffset) -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(stackObjectValue(offset.into_native())) }
    }

    pub fn stack_integer_value(offset: StackOffset) -> sqInt {
        unsafe { stackIntegerValue(offset.into_native()) }
    }

    pub fn primitive_fail() {
        unsafe { primitiveFail() };
    }

    pub fn primitive_fail_code(code: sqInt) {
        unsafe { primitiveFailFor(code) };
    }

    pub fn create_new_compiled_method(
        class: ObjectPointer,
        header: i64,
        bytecode_count: usize,
    ) -> ObjectPointer {
        let oop = unsafe {
            createNewMethodheaderbytecodeCount(
                class.into_native(),
                sqInt::try_from(header).unwrap(),
                sqInt::try_from(bytecode_count).unwrap(),
            )
        };
        ObjectPointer::from_native_c(oop)
    }

    pub fn method_return_value(value: ObjectPointer) {
        unsafe { methodReturnValue(value.into_native()) };
    }

    /// Return an item at an index within the indexable object (array, string, etc.).
    /// The index must start from 1, and not 0 like in Rust
    pub fn item_at(
        indexable_object: ObjectPointer,
        field_index: ObjectFieldIndex,
    ) -> ObjectPointer {
        unsafe {
            ObjectPointer::from_native_c(stObjectat(
                indexable_object.into_native(),
                field_index.into_native(),
            ))
        }
    }

    pub fn item_at_put(
        indexable_object: ObjectPointer,
        field_index: ObjectFieldIndex,
        value_object: ObjectPointer,
    ) -> ObjectPointer {
        unsafe {
            ObjectPointer::from_native_c(stObjectatput(
                indexable_object.into_native(),
                field_index.into_native(),
                value_object.into_native(),
            ))
        }
    }

    /// Return the size of an indexable object
    pub fn size_of(indexable_object: ObjectPointer) -> usize {
        (unsafe { stSizeOf(indexable_object.into_native()) }) as usize
    }

    pub fn object_field_at(object: ObjectPointer, field_index: ObjectFieldIndex) -> ObjectPointer {
        // watch out! the interpreter function expects to get field index first and object second
        unsafe {
            ObjectPointer::from_native_c(fetchPointerofObject(
                field_index.into_native(),
                object.into_native(),
            ))
        }
    }

    pub fn first_indexable_field(object: ObjectPointer) -> *mut c_void {
        unsafe { firstIndexableField(object.into_native()) }
    }

    pub fn bool_object(value: bool) -> ObjectPointer {
        if value {
            Self::true_object()
        } else {
            Self::false_object()
        }
    }

    pub fn true_object() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(trueObject()) }
    }

    pub fn false_object() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(falseObject()) }
    }

    pub fn nil_object() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(nilObject()) }
    }

    pub fn instantiate_class(class: ObjectPointer, is_pinned: bool) -> ObjectPointer {
        let is_pinned = if is_pinned { 1 } else { 0 };
        let oop = unsafe { instantiateClassisPinned(class.into_native(), is_pinned) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn instantiate_indexable_class_of_size(class: ObjectPointer, size: usize) -> ObjectPointer {
        let oop = unsafe { instantiateClassindexableSize(class.into_native(), size as sqInt) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn method_return_boolean(value: bool) {
        let boolean = Self::bool_object(value);
        unsafe { methodReturnBool(boolean.into_native()) };
    }

    pub fn method_return_integer(value: i64) {
        unsafe { methodReturnInteger(value) };
    }

    pub fn new_integer(number: impl Into<sqInt>) -> ObjectPointer {
        let oop = unsafe { integerObjectOf(number.into()) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn identity_hash(object: ObjectPointer) -> u32 {
        let hash = if Self::could_oop_be_class(object) {
            Self::behavior_identity_hash(object)
        } else {
            Self::object_identity_hash(object)
        };

        hash << 8
    }

    pub fn could_oop_be_class(object: ObjectPointer) -> bool {
        (unsafe { addressCouldBeClassObj(object.into_native()) }) != 0
    }

    fn object_identity_hash(object: ObjectPointer) -> u32 {
        unsafe { hashBitsOf(object.into_native()) as u32 }
    }

    fn behavior_identity_hash(object: ObjectPointer) -> u32 {
        unsafe { ensureBehaviorHash(object.into_native()) as u32 }
    }

    pub fn first_byte_pointer_of_data_object(object: ObjectPointer) -> *mut c_void {
        unsafe { firstBytePointerOfDataObject(object.into_native()) }
    }

    pub fn is_identical(first: ObjectPointer, second: ObjectPointer) -> Option<bool> {
        if Self::is_oop_forwarded(first) {
            return None;
        }

        if Self::is_oop_forwarded(second) {
            return None;
        }

        Some(first == second)
    }
    pub fn is_oop_forwarded(object: ObjectPointer) -> bool {
        (unsafe { isOopForwarded(object.into_native()) }) != 0
    }

    pub fn pointer_at_pointer(pointer: *mut c_void) -> *mut c_void {
        unsafe { *(pointer as *mut *mut c_void) }
    }
}
