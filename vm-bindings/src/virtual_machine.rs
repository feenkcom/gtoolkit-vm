use crate::bindings::{
    addressCouldBeClassObj, classArray, classExternalAddress, classString,
    createNewMethodheaderbytecodeCount, ensureBehaviorHash, falseObject, fetchClassOfNonImm,
    fetchPointerofObject, firstBytePointerOfDataObject, firstFixedField, firstIndexableField,
    getThisContext, hashBitsOf, instVarofContext, instantiateClassindexableSize,
    instantiateClassindexableSizeisPinned, instantiateClassisPinned, integerObjectOf, isOld,
    isOopForwarded, isYoung, methodArgumentCount, methodReturnBool, methodReturnInteger,
    methodReturnValue, nilObject, possibleOldObjectStoreInto, possiblePermObjectStoreIntovalue,
    primitiveFail, primitiveFailFor, sqInt, stContextSize, stObjectat, stObjectatput,
    stSizeOf, stackIntegerValue, stackValue, trueObject,
};
use crate::prelude::NativeTransmutable;
use crate::{ObjectFieldIndex, ObjectPointer, StackOffset};
use std::os::raw::c_void;
use vm_object_model::{AnyObjectRef, ObjectRef, RawObjectPointer};

pub struct Smalltalk {}

impl Smalltalk {
    pub fn new() -> Self {
        Self {}
    }

    pub fn method_argument_count() -> usize {
        unsafe { methodArgumentCount() as usize }
    }

    /// Get a value from the stack.
    /// It can be either an immediate value (integer, float, char) or
    /// an object.
    pub fn stack_value(offset: StackOffset) -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(stackValue(offset.into_native())) }
    }

    /// Get a reference to a value from the stack.
    /// It can be either an immediate value (integer, float, char) or
    /// an object.
    pub fn stack_ref(offset: StackOffset) -> AnyObjectRef {
        (unsafe { RawObjectPointer::new(stackValue(offset.into_native())) }).into()
    }

    /// Check if the value on a stack is an object (non-intermediate) and return it.
    pub fn stack_object_value(offset: StackOffset) -> Option<ObjectPointer> {
        let value = Self::stack_value(offset);
        if value.is_immediate() {
            None
        } else {
            Some(value)
        }
    }

    pub fn get_stack_value(&self, offset: StackOffset) -> AnyObjectRef {
        let value = Self::stack_value(offset);
        AnyObjectRef::from(RawObjectPointer::from(value.as_i64()))
    }

    /// Return an object on a stack. May return an invalid pointer if
    pub fn stack_object_value_unchecked(offset: StackOffset) -> ObjectPointer {
        let value = Self::stack_value(offset);
        if value.is_immediate() {
            Self::primitive_fail();
            0.into()
        } else {
            value
        }
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

    pub fn first_fixed_field(object: ObjectPointer) -> *mut c_void {
        unsafe { firstFixedField(object.into_native()) }
    }

    pub fn primitive_bool_object(value: bool) -> ObjectPointer {
        if value {
            Self::true_object()
        } else {
            Self::false_object()
        }
    }

    pub fn bool_object(value: bool) -> ObjectRef {
        AnyObjectRef::from(RawObjectPointer::new(
            Self::primitive_bool_object(value).as_i64(),
        ))
        .as_object()
        .unwrap()
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

    pub fn primitive_class_array() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(classArray()) }
    }

    pub fn class_array() -> ObjectRef {
        AnyObjectRef::from(RawObjectPointer::from(
            Self::primitive_class_array().into_native(),
        ))
        .as_object()
        .unwrap()
    }

    pub fn class_external_address() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(classExternalAddress()) }
    }

    pub fn class_string() -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(classString()) }
    }

    pub fn primitive_instantiate_class(class: ObjectPointer, is_pinned: bool) -> ObjectPointer {
        let is_pinned = if is_pinned { 1 } else { 0 };
        let oop = unsafe { instantiateClassisPinned(class.into_native(), is_pinned) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn instantiate_class(class: ObjectRef) -> AnyObjectRef {
        let pointer = Self::primitive_instantiate_class(
            ObjectPointer::from(class.into_inner().as_i64()),
            false,
        );
        AnyObjectRef::from(RawObjectPointer::from(pointer.into_native()))
    }

    pub fn instantiate<T: TryFrom<AnyObjectRef, Error = vm_object_model::Error>>(
        class: ObjectRef,
    ) -> vm_object_model::Result<T> {
        let object_ref = Self::instantiate_class(class);
        T::try_from(object_ref)
    }

    pub fn primitive_instantiate_indexable_class_of_size(
        class: ObjectPointer,
        size: usize,
    ) -> ObjectPointer {
        let oop = unsafe { instantiateClassindexableSize(class.into_native(), size as sqInt) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn primitive_instantiate_indexable_class_of_size_pinned(
        class: ObjectPointer,
        size: usize,
        is_pinned: bool,
    ) -> ObjectPointer {
        let is_pinned = if is_pinned { 1 } else { 0 };
        let oop = unsafe {
            instantiateClassindexableSizeisPinned(
                class.into_native(),
                size as sqInt,
                is_pinned as sqInt,
            )
        };
        ObjectPointer::from_native_c(oop)
    }

    pub fn instantiate_indexable_class(class: ObjectRef, size: usize) -> AnyObjectRef {
        let pointer = Self::primitive_instantiate_indexable_class_of_size(
            ObjectPointer::from(class.into_inner().as_i64()),
            size,
        );
        AnyObjectRef::from(RawObjectPointer::from(pointer.into_native()))
    }

    pub fn instantiate_indexable<T: TryFrom<AnyObjectRef, Error = vm_object_model::Error>>(
        class: ObjectRef,
        size: usize,
    ) -> vm_object_model::Result<T> {
        let object_ref = Self::instantiate_indexable_class(class, size);
        T::try_from(object_ref)
    }

    pub fn instantiate_indexable_class_of_size_pinned(
        class: ObjectPointer,
        size: usize,
        is_pinned: bool,
    ) -> ObjectPointer {
        let is_pinned = if is_pinned { 1 } else { 0 };
        let oop = unsafe {
            instantiateClassindexableSizeisPinned(
                class.into_native(),
                size as sqInt,
                is_pinned as sqInt,
            )
        };
        ObjectPointer::from_native_c(oop)
    }

    pub fn method_return_boolean(value: bool) {
        let boolean = Self::primitive_bool_object(value);
        unsafe { methodReturnBool(boolean.into_native()) };
    }

    pub fn method_return_integer(value: i64) {
        unsafe { methodReturnInteger(value) };
    }

    pub fn new_integer(number: impl Into<sqInt>) -> ObjectPointer {
        let oop = unsafe { integerObjectOf(number.into()) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn identity_hash(object: ObjectPointer) -> u64 {
        let hash = if Self::could_oop_be_class(object) {
            Self::behavior_identity_hash(object)
        } else {
            Self::object_identity_hash(object)
        };

        (hash as u64) << 8
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
        if Self::is_oop_forwarded(second) {
            return None;
        }

        Some(first == second)
    }
    pub fn is_oop_forwarded(object: ObjectPointer) -> bool {
        (unsafe { isOopForwarded(object.into_native()) }) != 0
    }

    pub fn is_old(object: ObjectPointer) -> bool {
        (unsafe { isOld(object.into_native()) }) != 0
    }

    pub fn is_young(object: ObjectPointer) -> bool {
        (unsafe { isYoung(object.into_native()) }) != 0
    }

    pub fn possible_old_object_store_into(object: ObjectPointer) {
        unsafe { possibleOldObjectStoreInto(object.into_native()) };
    }

    pub fn possible_perm_object_store_into(object: ObjectPointer, value: ObjectPointer) {
        unsafe { possiblePermObjectStoreIntovalue(object.into_native(), value.into_native()) };
    }

    pub fn pointer_at_pointer(pointer: *mut c_void) -> *mut c_void {
        unsafe { *(pointer as *mut *mut c_void) }
    }

    pub fn prepare_to_store(object: ObjectPointer, value: ObjectPointer) {
        if Smalltalk::is_old(object) && Smalltalk::is_young(value) {
            Smalltalk::possible_old_object_store_into(object);
        }
        Smalltalk::possible_perm_object_store_into(object, value);
    }

    pub fn this_context() -> ObjectRef {
        ObjectRef::try_from(RawObjectPointer::new(unsafe { getThisContext() })).unwrap()
    }

    pub fn context_sender(context: ObjectRef) -> ObjectRef {
        ObjectRef::try_from(RawObjectPointer::new(unsafe {
            instVarofContext(0, context.into_inner().as_i64())
        }))
        .unwrap()
    }

    pub fn context_method(context: ObjectRef) -> ObjectRef {
        ObjectRef::try_from(RawObjectPointer::new(unsafe {
            instVarofContext(3, context.into_inner().as_i64())
        }))
        .unwrap()
    }

    pub fn context_stack_length(context: ObjectRef) -> usize {
        let mut length = 1;
        let nil_object =
            ObjectRef::try_from(RawObjectPointer::new(Self::nil_object().into_native())).unwrap();

        let mut sender = context;
        while sender != nil_object {
            length += 1;
            sender = Self::context_sender(sender);
        }

        length
    }

    pub fn context_inst_var_at(context: ObjectRef, index: impl Into<ObjectFieldIndex>) -> AnyObjectRef {
        AnyObjectRef::from(RawObjectPointer::new(unsafe {
            instVarofContext(index.into().into_native(), context.into_inner().as_i64())
        }))
    }

    pub fn context_size(context: ObjectRef) -> usize {
        unsafe { stContextSize(context.into_inner().as_i64()) as usize }
    }

    pub fn context_at(context: ObjectRef, index: impl Into<ObjectFieldIndex>) -> AnyObjectRef {
        AnyObjectRef::from(RawObjectPointer::new(unsafe {
            stObjectat(context.into_inner().as_i64(), index.into().into_native())
        }))
    }

    pub fn class_of_object(object: ObjectRef) -> ObjectRef {
        ObjectRef::try_from(RawObjectPointer::new(unsafe {
            fetchClassOfNonImm(object.into_inner().as_i64())
        }))
        .unwrap()
    }
}
