use enum_display::EnumDisplay;
use std::convert::Infallible;
use std::ffi::OsString;
use std::num::TryFromIntError;
use std::ops::Deref;
use std::os::raw::*;
use vm_bindings::{ObjectPointer, Smalltalk};

use crate::objects::{ArrayRef, ByteStringRef, ExternalAddressRef};
use libffi::middle::{Arg, Cif, CodePtr, Type};
use libloading::Library;
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use thiserror::Error;
use vm_bindings::bindings::sqInt;
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct ExternalFunction {
    this: Object,
    callout: ExternalAddressRef,
    module_name: ByteStringRef,
    function_name: ByteStringRef,
    argument_types: ArrayRef,
    return_type: BareFFITypeRef,
    external_object_class: ObjectRef,
    external_enumeration_class: ObjectRef,
}

impl ExternalFunction {
    pub fn callout_mut(&mut self) -> &mut Callout {
        unsafe { &mut *(self.callout.read_address() as *mut Callout) }
    }

    fn invalidate(&mut self) -> Result<(), Error> {
        if self.callout.is_null() {
            let callout = self.new_callout()?;
            let callout_ptr = Box::into_raw(Box::new(callout));
            self.callout.set_address(callout_ptr as *mut c_void);
        }
        Ok(())
    }

    fn new_callout(&self) -> Result<Callout, Error> {
        let return_type = MarshallType::try_from(self.return_type)?;

        let mut argument_types = vec![MarshallType::Void; self.argument_types.len()];
        for (index, each_type) in self.argument_types.iter().enumerate() {
            let each_type = BareFFITypeRef::try_from(*each_type)?;
            let marshall_type = MarshallType::try_from(each_type)?;
            argument_types[index] = marshall_type;
        }

        Callout::new(
            OsString::from(self.module_name.to_string()),
            OsString::from(self.function_name.to_string()),
            argument_types,
            return_type,
        )
    }
}

#[derive(Debug)]
pub struct Callout {
    function_name: OsString,
    module_name: OsString,
    library: Library,
    function: CodePtr,
    argument_types: Vec<ArgumentType>,
    result: ReturnType,
    cif: Cif,
    // runtime state.
    // Since pharo is single threaded, we don't need to care about synchronization here
    marshalled_arguments: Vec<MarshalledValue>,
    arguments: Vec<Arg>,
}

impl Callout {
    pub fn new(
        module_name: OsString,
        function_name: OsString,
        argument_marshall_types: Vec<MarshallType>,
        return_type: MarshallType,
    ) -> Result<Self, Error> {
        let args = argument_marshall_types.iter().map(|each| Type::from(*each));
        let ret = Type::from(return_type);
        let cif = Cif::new(args, ret);

        let library = unsafe { Library::new(&module_name) }?;
        let function =
            unsafe { library.get::<unsafe extern "C" fn()>(function_name.as_encoded_bytes()) }?;
        let function = CodePtr::from_ptr(unsafe { function.into_raw() }.as_raw_ptr());

        let amount_of_arguments = argument_marshall_types.len();
        let mut argument_types = Vec::with_capacity(amount_of_arguments);
        for each in argument_marshall_types {
            let each_type = ArgumentType::new(each);
            argument_types.push(each_type);
        }

        Ok(Self {
            argument_types,
            result: ReturnType::new(return_type),
            cif,
            function_name,
            module_name,
            library,
            function,
            marshalled_arguments: vec![MarshalledValue::Void; amount_of_arguments],
            arguments: vec![Arg::new(&0); amount_of_arguments],
        })
    }

    pub fn call<T>(&self) -> T {
        unsafe { self.cif.call::<T>(self.function, self.arguments.as_slice()) }
    }
}

#[derive(Debug)]
pub struct ArgumentType {
    marshall_type: MarshallType,
    ffi_type: Type,
}

impl ArgumentType {
    pub fn new(marshall_type: MarshallType) -> Self {
        let ffi_type: Type = marshall_type.into();

        Self {
            marshall_type,
            ffi_type,
        }
    }
}

#[derive(Debug)]
pub struct ReturnType {
    marshall_type: MarshallType,
    ffi_type: Type,
}

impl ReturnType {
    pub fn new(marshall_type: MarshallType) -> Self {
        Self {
            marshall_type,
            ffi_type: marshall_type.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Object model error")]
    ObjectModel(#[from] vm_object_model::Error),
    #[error("Primitive conversion error")]
    PrimitiveTryFrom(#[from] TryFromIntError),
    #[error("Primitive conversion error")]
    Infallible(#[from] Infallible),
    #[error("Wrong number of arguments: {0}")]
    WrongNumberOfArguments(usize),
    #[error("LibLoading error")]
    LibLoading(#[from] libloading::Error),
    #[error("{0} can't be used as an argument type")]
    IllegalArgumentType(MarshallType),
    #[error("{0} is an invalid marshall type")]
    InvalidMarshallType(i64),
    #[error("Not a float")]
    InstanceNotFloat,
    #[error("Not an integer")]
    InstanceNotInteger,
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveBareFfiCallout() -> Result<(), Error> {
    let mut external_function =
        unsafe { ExternalFunctionRef::from_any_object_unchecked(Smalltalk::method_receiver()) };
    external_function.invalidate()?;

    let mut marshaller = ArgumentMarshall {
        external_object_class: external_function.external_object_class,
        external_enumeration_class: external_function.external_enumeration_class,
    };

    let callout = external_function.callout_mut();

    let amount_of_function_arguments = Smalltalk::method_argument_count();
    if amount_of_function_arguments != callout.argument_types.len() {
        Err(Error::WrongNumberOfArguments(amount_of_function_arguments))?;
    }

    for (each_argument_index, each_arg_type) in callout.argument_types.iter().enumerate() {
        let each_object = Smalltalk::get_method_argument(each_argument_index);
        let each_arg = marshaller.marshall(each_object, each_arg_type)?;

        callout.marshalled_arguments[each_argument_index] = each_arg;
        callout.arguments[each_argument_index] =
            callout.marshalled_arguments[each_argument_index].as_arg();
    }

    let result = call_and_marshall_result(callout)?;
    Smalltalk::method_return_value(ObjectPointer::from(result.as_i64()));

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveBareFfiCalloutInvalidate() -> Result<(), Error> {
    let mut external_function = ExternalFunctionRef::try_from(Smalltalk::method_receiver())?;
    external_function.invalidate()?;

    println!("{:#?}", external_function.callout_mut());

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveBareFfiCalloutRelease() -> Result<(), Error> {
    let callout_external_address = Smalltalk::get_method_argument(0).as_object()?;
    let callout_ptr = Smalltalk::read_external_address(callout_external_address);
    if callout_ptr.is_null() {
        return Ok(());
    }

    let callout = unsafe { Box::from_raw(callout_ptr as *mut Callout) };
    drop(callout);
    Ok(())
}

struct ArgumentMarshall {
    external_object_class: ObjectRef,
    external_enumeration_class: ObjectRef,
}

impl ArgumentMarshall {
    fn marshall(&self, object: AnyObjectRef, ty: &ArgumentType) -> Result<MarshalledValue, Error> {
        match ty.marshall_type {
            MarshallType::Void => Err(Error::IllegalArgumentType(ty.marshall_type)),
            MarshallType::Bool => Ok(MarshalledValue::Bool(
                Smalltalk::true_object() == ObjectPointer::from(object.as_i64()),
            )),
            MarshallType::U8 => self.try_marshall_as_rust_integer(object, MarshalledValue::U8),
            MarshallType::I8 => self.try_marshall_as_rust_integer(object, MarshalledValue::I8),
            MarshallType::U16 => self.try_marshall_as_rust_integer(object, MarshalledValue::U16),
            MarshallType::I16 => self.try_marshall_as_rust_integer(object, MarshalledValue::I16),
            MarshallType::U32 => self.try_marshall_as_rust_integer(object, MarshalledValue::U32),
            MarshallType::I32 => self.try_marshall_as_rust_integer(object, MarshalledValue::I32),
            MarshallType::U64 => self.try_marshall_as_rust_integer(object, MarshalledValue::U64),
            MarshallType::I64 => self.try_marshall_as_rust_integer(object, MarshalledValue::I64),
            MarshallType::USize => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::USize)
            }
            MarshallType::ISize => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::ISize)
            }
            MarshallType::SChar => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::SChar)
            }
            MarshallType::UChar => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::UChar)
            }
            MarshallType::Short => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::Short)
            }
            MarshallType::UShort => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::UShort)
            }
            MarshallType::Int => self.try_marshall_as_rust_integer(object, MarshalledValue::Int),
            MarshallType::UInt => self.try_marshall_as_rust_integer(object, MarshalledValue::UInt),
            MarshallType::Long => self.try_marshall_as_rust_integer(object, MarshalledValue::Long),
            MarshallType::ULong => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::ULong)
            }
            MarshallType::LongLong => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::LongLong)
            }
            MarshallType::ULongLong => {
                self.try_marshall_as_rust_integer(object, MarshalledValue::ULongLong)
            }
            MarshallType::F32 => {
                if object.is_immediate() {
                    let immediate = object.as_immediate()?;
                    if let Some(integer) = immediate.as_integer() {
                        return Ok(MarshalledValue::F32(integer as f32));
                    }
                }
                if Smalltalk::is_float(object) {
                    return Ok(MarshalledValue::F32(
                        Smalltalk::float_value_of(object) as f32
                    ));
                }
                Err(Error::InstanceNotFloat)
            }
            MarshallType::F64 => {
                if object.is_immediate() {
                    let immediate = object.as_immediate()?;
                    if let Some(integer) = immediate.as_integer() {
                        return Ok(MarshalledValue::F64(integer as f64));
                    }
                }
                if Smalltalk::is_float(object) {
                    return Ok(MarshalledValue::F64(Smalltalk::float_value_of(object)));
                }
                Err(Error::InstanceNotFloat)
            }
            MarshallType::Pointer => {
                let ptr = marshall_object_as_pointer(object, self.external_object_class)?;
                Ok(MarshalledValue::Pointer(ptr))
            }
        }
    }

    fn try_marshall_as_rust_integer<T: TryFrom<i64>, F>(
        &self,
        object: AnyObjectRef,
        variant: F,
    ) -> Result<MarshalledValue, Error>
    where
        F: Fn(T) -> MarshalledValue,
    {
        match object.as_immediate() {
            Ok(immediate) => {
                let value: T = immediate
                    .try_as_integer()?
                    .try_into()
                    .map_err(|error| Error::InstanceNotInteger)?;
                Ok(variant(value))
            }
            Err(error) => {
                if Smalltalk::is_kind_of(object, self.external_enumeration_class) {
                    let object = object.as_object()?;
                    let value = object
                        .inst_var_at(0)
                        .ok_or_else(|| Error::WrongNumberOfArguments(0))?;

                    self.try_marshall_as_rust_integer(value, variant)
                } else {
                    Err(error)?
                }
            }
        }
    }
}

fn marshall_object_as_pointer(
    any_object: AnyObjectRef,
    external_object_class: ObjectRef,
) -> Result<*const c_void, Error> {
    if Smalltalk::nil_object() == any_object {
        return Ok(std::ptr::null());
    }
    
    let object = any_object.as_object()?;

    if Smalltalk::class_of_object(object) == Smalltalk::class_external_address() {
        return Ok(Smalltalk::read_external_address(object) as *const c_void);
    }

    if Smalltalk::is_kind_of(any_object, external_object_class) {
        let handle = object
            .inst_var_at(0)
            .ok_or_else(|| Error::WrongNumberOfArguments(0))?;
        return marshall_object_as_pointer(handle, external_object_class);
    }

    Ok(object.first_fixed_field_ptr())
}

macro_rules! call_and_marshall_integer {
    ($callout:expr, $ty:ty) => {{
        let value = $callout.call::<$ty>();
        Ok(Smalltalk::new_integer(value))
    }};
}

macro_rules! call_and_try_marshall_integer {
    ($callout:expr, $ty:ty) => {{
        let value: sqInt = $callout.call::<$ty>().try_into()?;
        Ok(Smalltalk::new_integer(value))
    }};
}

fn call_and_marshall_result(callout: &Callout) -> Result<AnyObjectRef, Error> {
    match callout.result.marshall_type {
        MarshallType::Void => {
            callout.call::<c_void>();
            Ok(Smalltalk::nil_object())
        }
        MarshallType::Bool => {
            let result = callout.call::<bool>();
            Ok(Smalltalk::bool_object(result).into())
        }
        MarshallType::U8 => {
            call_and_marshall_integer!(callout, u8)
        }
        MarshallType::I8 => {
            call_and_marshall_integer!(callout, i8)
        }
        MarshallType::U16 => {
            call_and_marshall_integer!(callout, u16)
        }
        MarshallType::I16 => {
            call_and_marshall_integer!(callout, i16)
        }
        MarshallType::U32 => {
            call_and_try_marshall_integer!(callout, u32)
        }
        MarshallType::I32 => {
            call_and_marshall_integer!(callout, i32)
        }
        MarshallType::U64 => {
            call_and_try_marshall_integer!(callout, u64)
        }
        MarshallType::I64 => {
            call_and_try_marshall_integer!(callout, i64)
        }
        MarshallType::USize => {
            call_and_try_marshall_integer!(callout, usize)
        }
        MarshallType::ISize => {
            call_and_try_marshall_integer!(callout, isize)
        }
        MarshallType::SChar => {
            call_and_try_marshall_integer!(callout, c_schar)
        }
        MarshallType::UChar => {
            call_and_try_marshall_integer!(callout, c_uchar)
        }
        MarshallType::Short => {
            call_and_try_marshall_integer!(callout, c_short)
        }
        MarshallType::UShort => {
            call_and_try_marshall_integer!(callout, c_ushort)
        }
        MarshallType::Int => {
            call_and_try_marshall_integer!(callout, c_int)
        }
        MarshallType::UInt => {
            call_and_try_marshall_integer!(callout, c_uint)
        }
        MarshallType::Long => {
            call_and_try_marshall_integer!(callout, c_long)
        }
        MarshallType::ULong => {
            call_and_try_marshall_integer!(callout, c_ulong)
        }
        MarshallType::LongLong => {
            call_and_try_marshall_integer!(callout, c_longlong)
        }
        MarshallType::ULongLong => {
            call_and_try_marshall_integer!(callout, c_longlong)
        }
        MarshallType::F32 => {
            let result = callout.call::<f32>();
            Ok(Smalltalk::float_object_of(result as f64))
        }
        MarshallType::F64 => {
            let result = callout.call::<f64>();
            Ok(Smalltalk::float_object_of(result))
        }
        MarshallType::Pointer => {
            let result = callout.call::<*mut c_void>();
            Ok(AnyObjectRef::from(RawObjectPointer::from(
                Smalltalk::new_external_address(result).as_i64(),
            )))
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, EnumDisplay)]
#[repr(u8)]
pub enum MarshallType {
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    USize,
    ISize,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    F32,
    F64,
    Pointer,
}

#[derive(Debug, Copy, Clone)]
pub enum MarshalledValue {
    Void,
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    USize(usize),
    ISize(isize),
    SChar(c_schar),
    UChar(c_uchar),
    Short(c_short),
    UShort(c_ushort),
    Int(c_int),
    UInt(c_uint),
    Long(c_long),
    ULong(c_ulong),
    LongLong(c_longlong),
    ULongLong(c_ulonglong),
    F32(f32),
    F64(f64),
    Pointer(*const c_void),
}

impl MarshalledValue {
    pub fn as_arg(&self) -> Arg {
        match self {
            MarshalledValue::Void => {
                panic!("Void can't be an argument")
            }
            MarshalledValue::Bool(v) => Arg::new(v),
            MarshalledValue::U8(v) => Arg::new(v),
            MarshalledValue::I8(v) => Arg::new(v),
            MarshalledValue::U16(v) => Arg::new(v),
            MarshalledValue::I16(v) => Arg::new(v),
            MarshalledValue::U32(v) => Arg::new(v),
            MarshalledValue::I32(v) => Arg::new(v),
            MarshalledValue::U64(v) => Arg::new(v),
            MarshalledValue::I64(v) => Arg::new(v),
            MarshalledValue::USize(v) => Arg::new(v),
            MarshalledValue::ISize(v) => Arg::new(v),
            MarshalledValue::SChar(v) => Arg::new(v),
            MarshalledValue::UChar(v) => Arg::new(v),
            MarshalledValue::Short(v) => Arg::new(v),
            MarshalledValue::UShort(v) => Arg::new(v),
            MarshalledValue::Int(v) => Arg::new(v),
            MarshalledValue::UInt(v) => Arg::new(v),
            MarshalledValue::Long(v) => Arg::new(v),
            MarshalledValue::ULong(v) => Arg::new(v),
            MarshalledValue::LongLong(v) => Arg::new(v),
            MarshalledValue::ULongLong(v) => Arg::new(v),
            MarshalledValue::F32(v) => Arg::new(v),
            MarshalledValue::F64(v) => Arg::new(v),
            MarshalledValue::Pointer(v) => Arg::new(v),
        }
    }
}

impl From<MarshallType> for Type {
    fn from(value: MarshallType) -> Self {
        match value {
            MarshallType::Void => Self::void(),
            MarshallType::Bool => Self::u8(),
            MarshallType::U8 => Self::u8(),
            MarshallType::I8 => Self::i8(),
            MarshallType::U16 => Self::u16(),
            MarshallType::I16 => Self::i16(),
            MarshallType::U32 => Self::u32(),
            MarshallType::I32 => Self::i32(),
            MarshallType::U64 => Self::u64(),
            MarshallType::I64 => Self::i64(),
            MarshallType::USize => Self::usize(),
            MarshallType::ISize => Self::isize(),
            MarshallType::SChar => Self::c_schar(),
            MarshallType::UChar => Self::c_uchar(),
            MarshallType::Short => Self::c_short(),
            MarshallType::UShort => Self::c_ushort(),
            MarshallType::Int => Self::c_int(),
            MarshallType::UInt => Self::c_uint(),
            MarshallType::Long => Self::c_long(),
            MarshallType::ULong => Self::c_ulong(),
            MarshallType::LongLong => Self::c_longlong(),
            MarshallType::ULongLong => Self::c_ulonglong(),
            MarshallType::F32 => Self::f32(),
            MarshallType::F64 => Self::f64(),
            MarshallType::Pointer => Self::pointer(),
        }
    }
}

impl TryFrom<u8> for MarshallType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        MarshallType::from_u8(value)
            .ok_or_else(|| Error::InvalidMarshallType(value as _))
            .into()
    }
}

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct BareFFIType {
    this: Object,
    value: Immediate,
    name: ByteStringRef,
}

impl BareFFIType {
    pub fn value(&self) -> u8 {
        self.value.try_as_integer().unwrap().to_u8().unwrap()
    }
}

impl TryFrom<BareFFITypeRef> for MarshallType {
    type Error = Error;

    fn try_from(ty: BareFFITypeRef) -> Result<Self, Self::Error> {
        let int = ty.value.try_as_integer()?;
        let value: u8 = int.try_into()?;
        Ok(value.try_into()?)
    }
}
