extern crate bindgen;
extern crate cmake;
extern crate fs_extra;
extern crate regex;
extern crate titlecase;
extern crate which;

mod build_support;

use crate::build_support::{Builder, PlatformBuilder};

fn main() {
    let builder = Box::new(PlatformBuilder::default());
    println!("About to build a vm using {:?}", &builder);
    builder.ensure_build_tools();

    if !builder.is_compiled() {
        builder.generate_sources();
        builder.compile_sources();
    }

    if !builder.is_compiled() {
        panic!("Failed to compile {}", builder.vm_binary())
    }

    builder.link_libraries();
    builder.generate_bindings();
    builder.export_shared_libraries();
}
