use std::ffi::c_void;
use std::os::raw::c_char;

#[no_mangle]
pub fn pass_and_return_bool(boolean: bool) -> bool {
    return boolean;
}

#[no_mangle]
pub fn pass_and_return_i8(number: i8) -> i8 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_u8(number: u8) -> u8 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_i16(number: i16) -> i16 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_u16(number: u16) -> u16 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_i32(number: i32) -> i32 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_u32(number: u32) -> u32 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_f32(number: f32) -> f32 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_f64(number: f64) -> f64 {
    return number;
}

#[no_mangle]
pub fn pass_and_return_pointer(ptr: *const c_void) -> *const c_void {
    return ptr;
}

#[no_mangle]
pub fn pass_and_return_string(ptr: *const c_char) -> *const c_char {
    return ptr;
}