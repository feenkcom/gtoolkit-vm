extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate regex;
extern crate titlecase;
extern crate which;

mod build_support;

use crate::build_support::{Builder, PlatformBuilder};

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn main() {
    let builder = Box::new(PlatformBuilder::default());
    println!("About to build a vm using {:?}", &builder);
    builder.ensure_build_tools();

    if !builder.is_compiled() {
        builder.compile_sources();
    }

    if !builder.is_compiled() {
        panic!("Failed to compile {:?}", builder.vm_binary().display())
    }

    builder.link_libraries();
    builder.generate_bindings();
    builder.export_shared_libraries();
}
