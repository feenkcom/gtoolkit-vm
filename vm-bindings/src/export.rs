use crate::bindings::sqExport;

use crate::prelude::{Handle, NativeAccess, NativeClone, NativeDrop};
use std::ffi::{CStr, CString, FromVecWithNulError};
use std::fmt::{Debug, Formatter};
use std::os::raw::{c_char, c_void};

pub type NamedPrimitive = Handle<sqExport>;

#[macro_export]
macro_rules! primitive {
    ($func_name:ident) => {{
        let mut primitive_name = Vec::new();
        primitive_name.extend_from_slice(stringify!($func_name).as_bytes());
        // spur embeds accessorDepth after a primitive name
        primitive_name.extend_from_slice(b"\x00\xff");

        NamedPrimitive::new()
            .with_plugin_name("")
            .with_primitive_name_bytes(primitive_name)
            .with_primitive_address($func_name as *const std::os::raw::c_void)
    }};
}

#[macro_export]
macro_rules! try_primitive {
    ($func_name:ident) => {
        paste::item! {
            {
                #[no_mangle]
                #[allow(non_snake_case)]
                pub extern "C" fn [< try_ $func_name >]() {
                    $func_name().unwrap_or_else(|error| {
                        use user_error::{UserFacingError, UFE};
                        let error: Box<dyn std::error::Error> = error.into();
                        let user_facing_error: UserFacingError = error.into();

                        let short_text = user_facing_error.summary();
                        let long_text = user_facing_error
                            .reasons()
                            .map_or_else(
                                || user_facing_error.helptext(),
                                |reasons| Some(reasons.join("\n")),
                            )
                            .unwrap_or_else(|| "".to_string());

                        error!("{}", short_text);
                        error!("{}", long_text);
                        Smalltalk::primitive_fail()
                    });
                }
                NamedPrimitive::new()
                    .with_plugin_name("")
                    .with_primitive_name(concat!(stringify!($func_name)))
                    .with_primitive_address([< try_ $func_name >] as *const std::os::raw::c_void)
            }
        }
    };
}

impl NamedPrimitive {
    pub fn new() -> Self {
        Self::from_native_c(sqExport::new())
    }

    pub fn null() -> Self {
        Self::from_native_c(sqExport::new())
    }

    pub fn plugin_name(&self) -> &str {
        self.native().plugin_name()
    }

    pub fn with_plugin_name(mut self, name: impl Into<String>) -> Self {
        self.native_mut().set_plugin_name(name.into());
        self
    }

    pub fn primitive_name(&self) -> &str {
        self.native().primitive_name()
    }

    pub fn with_primitive_name(mut self, name: impl Into<String>) -> Self {
        self.native_mut().set_primitive_name(name.into());
        self
    }

    pub fn with_primitive_name_bytes(mut self, name: impl Into<Vec<u8>>) -> Self {
        self.native_mut().set_primitive_name_bytes(name);
        self
    }

    pub fn primitive_address(&self) -> *const c_void {
        self.native().primitive_address()
    }

    pub fn with_primitive_address(mut self, address: impl Into<*const c_void>) -> Self {
        self.native_mut().set_primitive_address(address.into());
        self
    }

    pub(crate) fn detect_exports_length(exports: *const NamedPrimitive) -> usize {
        let exports = exports as *const sqExport;

        let mut length = 0 as usize;
        loop {
            let each_export_ptr = unsafe { exports.offset(length as isize) };
            if each_export_ptr == std::ptr::null_mut() {
                break;
            }
            let each_export: &sqExport = unsafe { &*each_export_ptr };
            if !each_export.is_valid() {
                break;
            }
            length = length + 1;
        }
        length
    }
}

impl sqExport {
    fn new() -> Self {
        Self {
            pluginName: std::ptr::null_mut(),
            primitiveName: std::ptr::null_mut(),
            primitiveAddress: std::ptr::null_mut(),
        }
    }

    fn plugin_name(&self) -> &str {
        let plugin_name_ptr: *mut std::os::raw::c_char = self.pluginName;
        if plugin_name_ptr.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(plugin_name_ptr) }.to_str().unwrap()
    }

    fn set_plugin_name(&mut self, name: String) {
        if !self.pluginName.is_null() {
            panic!("Can't change plugin name if it is already assigned");
        }

        let plugin_name = CString::new(name).unwrap();
        let plugin_name_ptr = plugin_name.as_ptr() as *mut c_char;
        std::mem::forget(plugin_name);
        self.pluginName = plugin_name_ptr;
    }

    fn primitive_name(&self) -> &str {
        let primitive_name_ptr: *mut std::os::raw::c_char = self.primitiveName;
        if primitive_name_ptr.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(primitive_name_ptr) }
            .to_str()
            .unwrap()
    }

    fn set_primitive_name(&mut self, name: String) {
        if !self.primitiveName.is_null() {
            panic!("Can't change primitive name if it is already assigned");
        }

        let primitive_name = CString::new(name).unwrap();
        let primitive_name_ptr = primitive_name.as_ptr() as *mut c_char;
        std::mem::forget(primitive_name);
        self.primitiveName = primitive_name_ptr;
    }

    fn set_primitive_name_bytes(&mut self, name: impl Into<Vec<u8>>) {
        if !self.primitiveName.is_null() {
            panic!("Can't change primitive name if it is already assigned");
        }

        let primitive_name = unsafe { CString::from_vec_with_nul_unchecked(name.into()) };
        let primitive_name_ptr = primitive_name.as_ptr() as *mut c_char;
        std::mem::forget(primitive_name);
        self.primitiveName = primitive_name_ptr;
    }

    fn primitive_address(&self) -> *const std::os::raw::c_void {
        self.primitiveAddress
    }

    fn set_primitive_address(&mut self, address: *const std::os::raw::c_void) {
        self.primitiveAddress = address as *mut std::os::raw::c_void;
    }

    pub(crate) fn is_valid(&self) -> bool {
        if self.primitiveName.is_null() {
            return false;
        }

        if self.pluginName.is_null() {
            return false;
        }

        if self.primitiveAddress.is_null() {
            return false;
        }
        true
    }
}

impl NativeDrop for sqExport {
    fn drop(&mut self) {}
}

impl Debug for NamedPrimitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Export")
            .field("plugin_name", &self.plugin_name())
            .field("plugin_name (ptr)", &self.native().pluginName)
            .field("primitive_name", &self.primitive_name())
            .field("primitive_address", &self.primitive_address())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sq_export() {
        let mut sq_export = sqExport::new();
        assert_eq!(sq_export.pluginName.is_null(), true);
        assert_eq!(sq_export.primitiveName.is_null(), true);
        assert_eq!(sq_export.primitiveAddress.is_null(), true);
        assert_eq!(sq_export.plugin_name(), "");
        assert_eq!(sq_export.primitive_name(), "");
        assert_eq!(sq_export.primitive_address().is_null(), true);
        assert_eq!(sq_export.is_valid(), false);
    }

    #[test]
    fn new_export() {
        let mut export = NamedPrimitive::new();
        assert_eq!(export.plugin_name(), "");
        assert_eq!(export.primitive_name(), "");
        assert_eq!(export.primitive_address().is_null(), true);
    }

    #[test]
    fn export_with_plugin_name() {
        let mut export = NamedPrimitive::new();
        export = export.with_plugin_name("MyPlugin");
        assert_eq!(export.plugin_name(), "MyPlugin");
        assert_eq!(export.primitive_name(), "");
        assert_eq!(export.primitive_address().is_null(), true);
    }

    #[test]
    fn export_with_primitive_name() {
        let mut export = NamedPrimitive::new();
        export = export.with_primitive_name("myPrimitive");
        assert_eq!(export.plugin_name(), "");
        assert_eq!(export.primitive_name(), "myPrimitive");
        assert_eq!(export.primitive_address().is_null(), true);
    }
}
