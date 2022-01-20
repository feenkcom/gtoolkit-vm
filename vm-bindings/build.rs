extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;

use crate::build_support::*;
use user_error::{UserFacingError, UFE};

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn build() -> anyhow::Result<()> {
    let vm = VirtualMachine::new()?;
    vm.compile();
    Ok(())
}

fn main() {
    if let Err(error) = build() {
        let std_error: Box<dyn std::error::Error> = error.into();
        let user_facing_error: UserFacingError = std_error.into();
        user_facing_error.print_and_exit();
    }
}
