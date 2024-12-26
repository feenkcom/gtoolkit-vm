use crate::bindings::sqInt;

use crate::{Smalltalk, InterpreterProxy, ObjectFieldIndex, ObjectPointer};

#[cfg(feature = "ffi")]
use libffi::low::ffi_type;
#[cfg(feature = "ffi")]
use libffi_sys::*;

use crate::interpreter_proxy::write_value;
use crate::prelude::NativeTransmutable;
use anyhow::{bail, Result};
use std::os::raw::{c_double, c_float, c_void};

pub trait Marshallable {
    #[cfg(feature = "ffi")]
    fn marshall_argument_from_at_index_into_of_type_with_size(
        &self,
        arguments: ObjectPointer,
        index: usize,
        arg_type: &mut ffi_type,
    ) -> Result<*mut c_void>;

    #[cfg(feature = "ffi")]
    fn marshall_and_push_return_value_of_type_popping(
        &self,
        return_holder: *const c_void,
        return_type: &ffi_type,
        primitive_arguments_and_receiver_count: usize,
    ) -> Result<()>;

    fn marshall_float_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_double_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_u8_at(&self, array: ObjectPointer, index: usize, holder: *mut c_void)
        -> Result<()>;

    fn marshall_i8_at(&self, array: ObjectPointer, index: usize, holder: *mut c_void)
        -> Result<()>;

    fn marshall_u16_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_i16_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_u32_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;
    fn marshall_i32_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_u64_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;

    fn marshall_pointer_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()>;
}

impl Marshallable for InterpreterProxy {
    #[cfg(feature = "ffi")]
    fn marshall_argument_from_at_index_into_of_type_with_size(
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
            FFI_TYPE_SINT8 => self.marshall_i8_at(arguments, index, arg_holder)?,
            FFI_TYPE_UINT16 => self.marshall_u16_at(arguments, index, arg_holder)?,
            FFI_TYPE_SINT16 => self.marshall_i16_at(arguments, index, arg_holder)?,
            FFI_TYPE_UINT32 => self.marshall_u32_at(arguments, index, arg_holder)?,
            FFI_TYPE_SINT32 => self.marshall_i32_at(arguments, index, arg_holder)?,
            FFI_TYPE_UINT64 => self.marshall_u64_at(arguments, index, arg_holder)?,
            FFI_TYPE_SINT64 => {
                bail!(
                    "FFI_TYPE_SINT64 argument type of the argument at {} is not supported",
                    index
                );
            }
            FFI_TYPE_STRUCT => {
                bail!(
                    "FFI_TYPE_STRUCT argument type of the argument at {} is not supported",
                    index
                );
            }
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
    fn marshall_and_push_return_value_of_type_popping(
        &self,
        return_holder: *const c_void,
        return_type: &ffi_type,
        primitive_arguments_and_receiver_count: usize,
    ) -> Result<()> {
        match return_type.type_ as u32 {
            FFI_TYPE_FLOAT => {
                bail!("FFI_TYPE_FLOAT return type is not supported",);
            }
            FFI_TYPE_DOUBLE => {
                bail!("FFI_TYPE_DOUBLE return type is not supported",);
            }
            FFI_TYPE_UINT8 => {
                self.pop_then_push_integer(primitive_arguments_and_receiver_count, unsafe {
                    *(return_holder as *const u8)
                });
            }
            FFI_TYPE_SINT8 => {
                self.pop_then_push_integer(primitive_arguments_and_receiver_count, unsafe {
                    *(return_holder as *const i8)
                });
            }
            FFI_TYPE_UINT16 => {
                self.pop_then_push_integer(primitive_arguments_and_receiver_count, unsafe {
                    *(return_holder as *const u16)
                });
            }
            FFI_TYPE_SINT16 => {
                self.pop_then_push_integer(primitive_arguments_and_receiver_count, unsafe {
                    *(return_holder as *const i16)
                });
            }
            FFI_TYPE_UINT32 => {
                bail!("FFI_TYPE_UINT32 return type is not supported",);
            }
            FFI_TYPE_SINT32 => {
                bail!("FFI_TYPE_SINT32 return type is not supported",);
            }
            FFI_TYPE_UINT64 => {
                self.pop_then_push(
                    primitive_arguments_and_receiver_count,
                    self.new_positive_64bit_integer(unsafe { *(return_holder as *const u64) }),
                );
            }
            FFI_TYPE_SINT64 => {
                bail!("FFI_TYPE_SINT64 return type is not supported",);
            }
            FFI_TYPE_STRUCT => {
                bail!("FFI_TYPE_STRUCT return type is not supported",);
            }
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

    /// Reads the float value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the float value
    fn marshall_float_at(
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
    fn marshall_double_at(
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
    fn marshall_u8_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
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
                "unit8 argument ({}) at index {} exceeded the min value of uint8({})",
                value,
                index,
                u8::MIN
            );
        }

        write_value(value as u8, holder);
        Ok(())
    }

    /// Reads the int8 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the int8 value
    fn marshall_i8_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.integer_value_of(object);

        if value > i8::MAX as sqInt {
            bail!(
                "int8 argument ({}) at index {} exceeded the max value of int8({})",
                value,
                index,
                i8::MAX
            );
        }
        if value < i8::MIN as sqInt {
            bail!(
                "int8 argument ({}) at index {} exceeded the min value of int8({})",
                value,
                index,
                i8::MIN
            );
        }

        write_value(value as i8, holder);
        Ok(())
    }

    /// Reads the uint16 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the uint16 value
    fn marshall_u16_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.integer_value_of(object);
        if value > u16::MAX as sqInt {
            bail!(
                "uint16 argument ({}) at index {} exceeded the max value of uint16({})",
                value,
                index,
                u16::MAX
            );
        }
        if value < u16::MIN as sqInt {
            bail!(
                "unit16 argument ({}) at index {} exceeded the min value of uint16({})",
                value,
                index,
                u16::MIN
            );
        }

        write_value(value as u16, holder);
        Ok(())
    }

    /// Reads the int16 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the int16 value
    fn marshall_i16_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.integer_value_of(object);
        if value > i16::MAX as sqInt {
            bail!(
                "int16 argument ({}) at index {} exceeded the max value of int16({})",
                value,
                index,
                i16::MAX
            );
        }
        if value < i16::MIN as sqInt {
            bail!(
                "int16 argument ({}) at index {} exceeded the min value of int16({})",
                value,
                index,
                i16::MIN
            );
        }

        write_value(value as i16, holder);
        Ok(())
    }

    /// Reads the uint32 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the uint32 value
    fn marshall_u32_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.positive_32bit_value_of(object);
        write_value(value, holder);
        Ok(())
    }

    /// Reads the int32 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the int32 value
    fn marshall_i32_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.signed_32bit_value_of(object);
        write_value(value, holder);
        Ok(())
    }

    /// Reads the uint64 value from the array at a given index and store the value in a value holder at a given address.
    /// *Important!* The value holder must be already pre-allocated to fit the uint64 value
    fn marshall_u64_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let object = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));
        let value = self.positive_64bit_value_of(object);
        write_value(value, holder);
        Ok(())
    }

    fn marshall_pointer_at(
        &self,
        array: ObjectPointer,
        index: usize,
        holder: *mut c_void,
    ) -> Result<()> {
        let external_address = Smalltalk::object_field_at(array, ObjectFieldIndex::new(index));

        if !self.is_kind_of_class(external_address, self.class_external_address()) {
            bail!(
                "pointer argument at index {} is not an external address",
                index,
            );
        }

        let address = Smalltalk::object_field_at(external_address, ObjectFieldIndex::new(0));
        write_value(address.into_native(), holder);
        Ok(())
    }
}
