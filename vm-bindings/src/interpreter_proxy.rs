use crate::bindings::{
    calloc, exportClassOrNilAtIndex as classOrNilAtIndex, exportGetHandler as getHandler,
    exportReadAddress as readAddress, free, malloc, sqInt, VirtualMachine as sqInterpreterProxy,
};
use std::any::type_name;

use crate::prelude::{Handle, NativeAccess, NativeDrop, NativeTransmutable};
use crate::Smalltalk;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::mem::size_of;
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
        self.pop_then_push(amount_of_stack_items, Smalltalk::new_integer(number));
    }

    pub fn get_handler(&self, object: ObjectPointer) -> *mut c_void {
        unsafe { getHandler(object.into_native()) }
    }

    pub fn object_field_at_put(
        &self,
        object: ObjectPointer,
        field_index: ObjectFieldIndex,
        value: ObjectPointer,
    ) -> ObjectPointer {
        let function = self.native().storePointerofObjectwithValue.unwrap();
        // watch out! the interpreter function expects to get field index first and object second
        unsafe {
            ObjectPointer::from_native_c(function(
                field_index.into_native(),
                object.into_native(),
                value.into_native(),
            ))
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

    pub fn signed_32bit_value_of(&self, object: ObjectPointer) -> i32 {
        let function = self.native().signed32BitValueOf.unwrap();
        unsafe { function(object.into_native()) as i32 }
    }

    pub fn positive_64bit_value_of(&self, object: ObjectPointer) -> u64 {
        let function = self.native().positive64BitValueOf.unwrap();
        unsafe { cast_integer(function(object.into_native())) }
    }

    pub fn fetch_float_at(&self, object: ObjectPointer, index: ObjectFieldIndex) -> c_double {
        let function = self.native().fetchFloatofObject.unwrap();
        unsafe { function(index.into_native(), object.into_native()) }
    }

    pub fn signal_semaphore(&self, index: usize) {
        let function = self.native().signalSemaphoreWithIndex.unwrap();
        unsafe {
            function(index as sqInt);
        }
    }

    pub fn is_kind_of_class(&self, object: ObjectPointer, class: ObjectPointer) -> bool {
        let function = self.native().isKindOfClass.unwrap();
        unsafe { function(object.into_native(), class.into_native()) != 0 }
    }

    pub fn class_or_nil_at_index(&self, class_index: sqInt) -> ObjectPointer {
        unsafe { ObjectPointer::from_native_c(classOrNilAtIndex(class_index)) }
    }

    pub fn new_string(&self, string: impl AsRef<str>) -> ObjectPointer {
        let function = self.native().stringForCString.unwrap();
        let rust_str = string.as_ref();
        let c_string = CString::new(rust_str).unwrap();

        let oop = unsafe { function(c_string.as_ptr() as *mut c_char) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn new_string_from_cstring(&self, c_string: Option<&CStr>) -> ObjectPointer {
        if let Some(c_string) = c_string {
            let function = self.native().stringForCString.unwrap();
            let oop = unsafe { function(c_string.as_ptr() as *mut c_char) };
            ObjectPointer::from_native_c(oop)
        } else {
            Smalltalk::nil_object()
        }
    }

    pub fn new_external_address<T>(&self, address: *const T) -> ObjectPointer {
        let external_address = Smalltalk::primitive_instantiate_indexable_class_of_size(
            Smalltalk::class_external_address(),
            size_of::<*mut c_void>(),
        );
        unsafe {
            *(Smalltalk::first_indexable_field(external_address) as *mut *mut c_void) =
                address as *mut c_void
        };
        external_address
    }

    pub fn new_positive_64bit_integer(&self, integer: u64) -> ObjectPointer {
        let function = self.native().positive64BitIntegerFor.unwrap();
        let oop = unsafe { function(cast_integer(integer)) };
        ObjectPointer::from_native_c(oop)
    }

    pub fn read_address(&self, external_address_object: ObjectPointer) -> *mut c_void {
        unsafe { readAddress(external_address_object.into_native()) }
    }

    pub fn pin_object(&self, object: ObjectPointer) {
        let function = self.native().pinObject.unwrap();
        unsafe { function(object.into_native()) };
    }

    pub fn malloc(&self, bytes: usize) -> *mut c_void {
        unsafe { malloc(cast_integer(bytes)) }
    }

    pub fn calloc(&self, amount: usize, size: usize) -> *mut c_void {
        unsafe { calloc(cast_integer(amount), cast_integer(size)) }
    }

    pub fn free(&self, address: *mut c_void) {
        unsafe {
            free(address);
        }
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

fn cast_integer<T: Display + TryInto<R> + Copy, R: Display>(number: T) -> R {
    number.try_into().unwrap_or_else(|_| {
        panic!(
            "Failed to cast {} from {} to {}",
            number,
            type_name::<T>(),
            type_name::<R>()
        )
    })
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct ObjectPointer(sqInt);
impl NativeTransmutable<sqInt> for ObjectPointer {}
impl ObjectPointer {
    const NUM_SLOTS_MASK: u8 = 255;
    pub const TAG_MASK: sqInt = 7;

    pub fn as_i64(&self) -> i64 {
        self.0
    }

    pub fn read<T>(&self) -> T {
        let pointer: *mut T = unsafe { std::mem::transmute(self.into_native()) };
        unsafe { pointer.read_unaligned() }
    }

    pub fn read_long(&self) -> sqInt {
        self.read::<sqInt>()
    }

    pub fn read_u32(&self) -> u32 {
        self.read::<u32>()
    }

    pub fn read_u8(&self) -> u8 {
        self.read::<u8>()
    }

    pub fn offset_by(&self, offset: sqInt) -> Self {
        ObjectPointer(self.0 + offset)
    }

    pub fn is_immediate(&self) -> bool {
        (self.into_native() & Self::TAG_MASK) != 0
    }
}

impl From<ObjectPointer> for sqInt {
    fn from(value: ObjectPointer) -> Self {
        value.0
    }
}

impl From<sqInt> for ObjectPointer {
    fn from(value: sqInt) -> Self {
        Self(value)
    }
}

impl From<*const c_void> for ObjectPointer {
    fn from(value: *const c_void) -> Self {
        Self(sqInt::try_from(value as usize).unwrap())
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ObjectFieldIndex(sqInt);
impl NativeTransmutable<sqInt> for ObjectFieldIndex {}
impl ObjectFieldIndex {
    pub fn new(index: usize) -> Self {
        Self(index as sqInt)
    }
}

impl From<u32> for ObjectFieldIndex {
    fn from(value: u32) -> Self {
        Self::new(value as usize)
    }
}

impl From<usize> for ObjectFieldIndex {
    fn from(value: usize) -> Self {
        Self::new(value)
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
