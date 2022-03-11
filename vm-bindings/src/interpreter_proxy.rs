use crate::bindings::{
    calloc, exportGetHandler as getHandler,
    exportInstantiateClassIsPinned as instantiateClassIsPinned, exportReadAddress as readAddress,
    free, malloc, memcpy, sqInt, VirtualMachine as sqInterpreterProxy,
};

#[cfg(feature = "ffi")]
use libffi::low::ffi_type;
#[cfg(feature = "ffi")]
use libffi_sys::*;

use std::ffi::CString;
use std::mem::{size_of, transmute};

use crate::prelude::{Handle, NativeAccess, NativeDrop, NativeTransmutable};
use anyhow::{bail, Result};
use libffi::high::arg;
use log::{log, Level};
use std::os::raw::{c_char, c_double, c_float, c_int, c_ushort, c_void};

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

    pub fn read_address(&self, external_address_object: ObjectPointer) -> *mut c_void {
        unsafe { readAddress(external_address_object.into_native()) }
    }

    /// Reads the float value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the float value
    pub fn marshall_float_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let value = self.fetch_float_at(array, ObjectFieldIndex::new(index));
        if value > c_float::MAX as c_double {
            bail!(
                "Float argument ({}) at index {} exceeded the max value of c_float({})",
                value,
                index,
                c_float::MAX
            );
        }
        if value < c_float::MIN as c_double {
            bail!(
                "Float argument ({}) at index {} exceeded the min value of c_float({})",
                value,
                index,
                c_float::MIN
            );
        }
        write_value(value as c_float, holder);
        Ok(())
    }

    /// Reads the double value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the double value
    pub fn marshall_double_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let value = self.fetch_float_at(array, ObjectFieldIndex::new(index));
        write_value(value, holder);
        Ok(())
    }

    /// Reads the uint8 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the uint8 value
    pub fn marshall_u8_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = self.object_field_at(array, ObjectFieldIndex::new(index));
        let value = if self.is_character_object(object) {
            self.character_value_of(object) as sqInt
        } else {
            self.integer_value_of(object)
        };

        if value > u8::MAX as sqInt {
            bail!(
                "uint8 argument ({}) at index {} exceeded the max value of uint8({})",
                value,
                index,
                u8::MAX
            );
        }
        if value < u8::MIN as sqInt {
            bail!(
                "unit argument ({}) at index {} exceeded the min value of uint8({})",
                value,
                index,
                u8::MIN
            );
        }

        write_value(value as u8, holder);
        Ok(())
    }

    pub fn marshall_pointer_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let external_address = self.object_field_at(array, ObjectFieldIndex::new(index));

        if !self.is_kind_of_class(external_address, self.class_external_address()) {
            bail!(
                "pointer argument at index {} is not an external address",
                index,
            );
        }

        let address = self.object_field_at(external_address, ObjectFieldIndex::new(0));
        write_value(address.into_native(), holder);
        Ok(())
    }

    #[cfg(feature = "ffi")]
    pub fn marshall_argument_from_at_index_into_of_type_with_size(
        &self,
        arguments: ObjectPointer,
        index: usize,
        arg_type: &mut ffi_type,
    ) -> Result<*mut c_void> {
        let arg_holder = self.malloc(arg_type.size);

        match arg_type.type_ as u32 {
            FFI_TYPE_FLOAT => self.marshall_float_at(arguments, index, arg_holder)?,
            FFI_TYPE_DOUBLE => self.marshall_double_at(arguments, index, arg_holder)?,
            FFI_TYPE_UINT8 => self.marshall_u8_at(arguments, index, arg_holder)?,
            FFI_TYPE_SINT8 => {}
            FFI_TYPE_UINT16 => {}
            FFI_TYPE_SINT16 => {}
            FFI_TYPE_UINT32 => {}
            FFI_TYPE_SINT32 => {}
            FFI_TYPE_UINT64 => {}
            FFI_TYPE_SINT64 => {}
            FFI_TYPE_STRUCT => {}
            FFI_TYPE_POINTER => self.marshall_pointer_at(arguments, index, arg_holder)?,
            FFI_TYPE_VOID => {
                bail!(
                    "Void argument type of the argument at {} is not supported",
                    index
                );
            }
            FFI_TYPE_INT => {
                bail!(
                    "Int argument type of the argument at {} is not supported",
                    index
                );
            }
            FFI_TYPE_LONGDOUBLE => {
                bail!(
                    "Long argument type of the argument at {} is not supported",
                    index
                );
            }
            FFI_TYPE_COMPLEX => {
                bail!(
                    "Complex argument type of the argument at {} is not supported",
                    index
                );
            }
            _ => {
                bail!(
                    "Unknown type {} of the argument at {}",
                    arg_type.type_,
                    index
                );
            }
        };

        Ok(arg_holder)
    }

    //StackInterpreterPrimitives >> marshallArgumentFrom: argumentsArrayOop atIndex: i into: argHolder ofType: argType withSize: argTypeSize [
    //
    // 	<option: #FEATURE_FFI>
    // 	[ argType ]
    // 		caseOf:
    // 			{([ FFI_TYPE_POINTER ]
    // 				-> [ self marshallPointerFrom: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_STRUCT ]
    // 				-> [ self marshallStructFrom: argumentsArrayOop at: i into: argHolder withSize: argTypeSize ]).
    // 			([ FFI_TYPE_FLOAT ]
    // 				-> [ self marshallFloatFrom: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_DOUBLE ]
    // 				-> [ self marshallDoubleFrom: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_SINT8 ]
    // 				-> [ self marshallSInt8From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_UINT8 ]
    // 				-> [ self marshallUInt8From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_SINT16 ]
    // 				-> [ self marshallSInt16From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_UINT16 ]
    // 				-> [ self marshallUInt16From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_SINT32 ]
    // 				-> [ self marshallSInt32From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_UINT32 ]
    // 				-> [ self marshallUInt32From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_SINT64 ]
    // 				-> [ self marshallSInt64From: argumentsArrayOop at: i into: argHolder ]).
    // 			([ FFI_TYPE_UINT64 ]
    // 				-> [ self marshallUInt64From: argumentsArrayOop at: i into: argHolder ])}
    // 		otherwise: [ self primitiveFailFor: PrimErrBadArgument ]
    // ]

    #[cfg(feature = "ffi")]
    pub fn marshall_and_push_return_value_of_type_popping(
        &self,
        return_holder: *const c_void,
        return_type: &ffi_type,
        primitive_arguments_and_receiver_count: usize,
    ) -> Result<()> {
        match return_type.type_ as u32 {
            FFI_TYPE_FLOAT => {}
            FFI_TYPE_DOUBLE => {}
            FFI_TYPE_UINT8 => {}
            FFI_TYPE_SINT8 => {}
            FFI_TYPE_UINT16 => {}
            FFI_TYPE_SINT16 => {}
            FFI_TYPE_UINT32 => {}
            FFI_TYPE_SINT32 => {}
            FFI_TYPE_UINT64 => {}
            FFI_TYPE_SINT64 => {}
            FFI_TYPE_STRUCT => {}
            FFI_TYPE_POINTER => {
                let address = unsafe { *(return_holder as *const *const c_void) };
                self.pop_then_push(
                    primitive_arguments_and_receiver_count,
                    self.new_external_address(address),
                )
            }
            FFI_TYPE_VOID => {
                self.pop(primitive_arguments_and_receiver_count - 1);
            }
            FFI_TYPE_INT => {
                bail!("Int return type is not supported",);
            }
            FFI_TYPE_LONGDOUBLE => {
                bail!("Long return type is not supported",);
            }
            FFI_TYPE_COMPLEX => {
                bail!("Complex return type is not supported",);
            }
            _ => {
                bail!("Unknown return type {}", return_type.type_);
            }
        };

        Ok(())
    }

    // StackInterpreterPrimitives >> marshallAndPushReturnValueFrom: returnHolder ofType: ffiType poping: argumentsAndReceiverCount [
    //
    // 	<option: #FEATURE_FFI>
    // 	<var: #ffiType type: #'ffi_type *'>
    //
    // 	[ ffiType type ]
    // 		caseOf: {
    // 			[ FFI_TYPE_SINT8 ] 	-> [ self pop: argumentsAndReceiverCount thenPushInteger: (objectMemory readSINT8AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_SINT16 ] 	-> [ self pop: argumentsAndReceiverCount thenPushInteger: (objectMemory readSINT16AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_SINT32 ] 	-> [ self
    // 													pop: argumentsAndReceiverCount
    // 													thenPush: (objectMemory signed32BitIntegerFor: (objectMemory readSINT32AtPointer: returnHolder)) ].
    // 			[ FFI_TYPE_SINT64 ] 	-> [ self
    // 													pop: argumentsAndReceiverCount
    // 													thenPush: (objectMemory signed64BitIntegerFor: (objectMemory readSINT64AtPointer: returnHolder)) ].
    //
    // 			[ FFI_TYPE_UINT8 ] 	-> [ self pop: argumentsAndReceiverCount thenPushInteger: (objectMemory readUINT8AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_UINT16 ] 	-> [ self pop: argumentsAndReceiverCount thenPushInteger: (objectMemory readUINT16AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_UINT32 ] 	-> [ self
    // 													pop: argumentsAndReceiverCount
    // 													thenPush: (objectMemory positive32BitIntegerFor: (objectMemory readUINT32AtPointer: returnHolder)) ].
    // 			[ FFI_TYPE_UINT64 ] 	-> [ self
    // 													pop: argumentsAndReceiverCount
    // 													thenPush: (objectMemory positive64BitIntegerFor: (objectMemory readUINT64AtPointer: returnHolder)) ].
    //
    // 			[ FFI_TYPE_POINTER ] 	-> [ self pop: argumentsAndReceiverCount thenPush: (objectMemory newExternalAddressWithValue: (objectMemory readPointerAtPointer: returnHolder)) ].
    //
    // 			[ FFI_TYPE_STRUCT ] 	-> [ self pop: argumentsAndReceiverCount thenPush: (self newByteArrayWithStructContent: returnHolder size: ffiType size) ].
    //
    // 			[ FFI_TYPE_FLOAT ] 	-> [ self pop: argumentsAndReceiverCount thenPushFloat: (objectMemory readFloat32AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_DOUBLE ] 	-> [ self pop: argumentsAndReceiverCount thenPushFloat: (objectMemory readFloat64AtPointer: returnHolder) ].
    // 			[ FFI_TYPE_VOID ] 		-> [ self pop: argumentsAndReceiverCount - 1 "Pop the arguments leaving the receiver" ]}
    // 			otherwise: [ self primitiveFailFor: PrimErrBadArgument ]
    //
    // ]

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

fn write_value<T>(value: T, holder: *mut c_void) {
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
