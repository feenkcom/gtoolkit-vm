#![cfg(target_os = "macos")]

use core_foundation::base::{Boolean, CFIndex};
use core_foundation::bundle::{CFBundleCopyBundleURL, CFBundleGetMainBundle};
use core_foundation::error::CFErrorRef;
use core_foundation::url::{CFURLGetFileSystemRepresentation, CFURLRef};
use libc::{c_char, strlen, PATH_MAX};
use libloading::{Library, Symbol};
use std::path::PathBuf;

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use crate::{ApplicationError, Result};

/// Try to change the working directory back to the original location
pub fn un_translocate() -> Result<()> {
    let translocation = MacTranslocation::new()?;
    if let Some(original_location) = translocation.original_location()? {
        std::env::set_current_dir(
            original_location
                .parent()
                .ok_or_else(|| ApplicationError::NoParentDirectory(original_location.clone()))?,
        )?;
    }
    Ok(())
}

pub struct MacTranslocation {
    library: Library,
}

impl MacTranslocation {
    pub fn new() -> Result<Self> {
        unsafe {
            let security_lib =
                Library::new("/System/Library/Frameworks/Security.framework/Security")?;
            Ok(Self {
                library: security_lib,
            })
        }
    }

    pub fn is_translocated(&self) -> Result<bool> {
        unsafe {
            let is_translocated_fn: Symbol<
                unsafe extern "C" fn(CFURLRef, *mut bool, *mut CFErrorRef) -> bool,
            > = match self.library.get(b"SecTranslocateIsTranslocatedURL") {
                Ok(func) => func,
                Err(_) => {
                    return Ok(false);
                }
            };

            let bundle = CFBundleGetMainBundle();
            let url = CFBundleCopyBundleURL(bundle);
            let mut bool_is_translocated = false;

            if is_translocated_fn(
                url,
                &mut bool_is_translocated as *mut _,
                std::ptr::null_mut(),
            ) {
            } else {
                return Err(ApplicationError::FailedToDetectIfTranslocated);
            }
            Ok(bool_is_translocated)
        }
    }

    pub fn original_location(&self) -> Result<Option<PathBuf>> {
        unsafe {
            let original_path_fn: Symbol<
                unsafe extern "C" fn(CFURLRef, *mut CFErrorRef) -> CFURLRef,
            > = match self.library.get(b"SecTranslocateCreateOriginalPathForURL") {
                Ok(func) => func,
                Err(_) => {
                    return Ok(None);
                }
            };

            let bundle = CFBundleGetMainBundle();
            let url = CFBundleCopyBundleURL(bundle);

            if self.is_translocated()? {
                let original_url: CFURLRef = original_path_fn(url, std::ptr::null_mut());
                if original_url == std::ptr::null() {
                    return Err(ApplicationError::FailedToDetectOriginalTranslocatedPath);
                }

                let mut buf = [0u8; PATH_MAX as usize];
                let result = CFURLGetFileSystemRepresentation(
                    original_url,
                    true as Boolean,
                    buf.as_mut_ptr(),
                    buf.len() as CFIndex,
                );
                if result == false as Boolean {
                    return Err(ApplicationError::FailedToDetectOriginalTranslocatedPath);
                }
                let len = strlen(buf.as_ptr() as *const c_char);
                let path = OsStr::from_bytes(&buf[0..len]);
                return Ok(Some(PathBuf::from(path)));
            }

            Ok(None)
        }
    }
}
