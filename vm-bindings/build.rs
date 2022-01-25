extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;

use crate::build_support::*;
use console::Emoji;
use std::io::Write;
use user_error::{UserFacingError, UFE};

pub static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
pub static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
pub static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
pub static BUILDING: Emoji<'_, '_> = Emoji("🔨 ", "");
#[cfg(target_arch = "aarch64")]
pub static MACOSX_DEPLOYMENT_TARGET: &str = "11.0";
#[cfg(not(target_arch = "aarch64"))]
pub static MACOSX_DEPLOYMENT_TARGET: &str = "10.8";

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn build() -> anyhow::Result<()> {
    // let vm = VirtualMachine::new()?;
    // vm.compile();
    //
    // // export the vm info to json
    // let json = serde_json::to_string_pretty(&vm)?;
    // let file_path = vm.get_core().artefact_directory().join("vm-info.json");
    // let mut file = std::fs::File::create(file_path)?;
    // writeln!(&mut file, "{}", json).unwrap();

    let builder = VirtualMachine::builder()?;
    builder.link_libraries();
    builder.generate_bindings();

    Ok(())
}

fn main() {
    if let Err(error) = build() {
        let std_error: Box<dyn std::error::Error> = error.into();
        let user_facing_error: UserFacingError = std_error.into();
        user_facing_error.print_and_exit();
    }
}
