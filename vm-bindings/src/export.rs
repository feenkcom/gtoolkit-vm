use crate::bindings::sqExport;

use crate::prelude::{Handle, NativeAccess, NativeClone, NativeDrop};
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter};
use std::os::raw::{c_char, c_void};

pub type NamedPrimitive = Handle<sqExport>;

#[macro_export]
macro_rules! primitive {
    ($func_name:ident) => {
        NamedPrimitive::new()
            .with_plugin_name("")
            .with_primitive_name(stringify!($func_name))
            .with_primitive_address($func_name as *const std::os::raw::c_void)
    };
}

impl NamedPrimitive {
    pub fn new() -> Self {
        Self::from_native_c(sqExport::new())
            .with_plugin_name("")
            .with_primitive_name("")
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
        let previous_name = self.take_plugin_name();
        drop(previous_name);

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
        let previous_name = self.take_primitive_name();
        drop(previous_name);

        let primitive_name = CString::new(name).unwrap();
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

    /// Take the ownership over the plugin name
    fn take_plugin_name(&mut self) -> String {
        let plugin_name_ptr: *mut std::os::raw::c_char = self.pluginName;
        self.pluginName = std::ptr::null_mut();
        if plugin_name_ptr.is_null() {
            return "".to_string();
        }
        unsafe { CString::from_raw(plugin_name_ptr) }
            .into_string()
            .unwrap()
    }

    fn take_primitive_name(&mut self) -> String {
        let primitive_name_ptr: *mut std::os::raw::c_char = self.primitiveName;
        self.primitiveName = std::ptr::null_mut();
        if primitive_name_ptr.is_null() {
            return "".to_string();
        }
        unsafe { CString::from_raw(primitive_name_ptr) }
            .into_string()
            .unwrap()
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
    fn drop(&mut self) {
        self.take_plugin_name();
        self.take_primitive_name();
    }
}

impl NativeClone for sqExport {
    fn clone(&self) -> Self {
        let mut new_sq_export = sqExport::new();
        if !self.pluginName.is_null() {
            new_sq_export.set_plugin_name(self.plugin_name().to_string());
        }
        if !self.primitiveName.is_null() {
            new_sq_export.set_primitive_name(self.primitive_name().to_string());
        }
        new_sq_export.set_primitive_address(self.primitive_address());
        new_sq_export
    }
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
        assert_eq!(sq_export.take_plugin_name(), "".to_string());
        assert_eq!(sq_export.take_primitive_name(), "".to_string());
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
