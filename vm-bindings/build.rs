extern crate bindgen;
extern crate cmake;
extern crate fs_extra;
extern crate regex;

mod build_support;

use crate::build_support::{Builder, PlatformBuilder};

fn main() {
    let builder = Box::new(PlatformBuilder::default());

    if !builder.is_compiled() {
        builder.generate_sources();
        builder.compile_sources();
    }

    builder.link_libraries();
    builder.generate_bindings();
    builder.export_shared_libraries();
}
