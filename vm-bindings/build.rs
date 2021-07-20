extern crate bindgen;
extern crate cmake;
extern crate file_matcher;
extern crate fs_extra;
extern crate titlecase;
extern crate which;

mod build_support;
use build_support::*;

///
/// Possible parameters
///  - VM_CLIENT_VMMAKER to use a specific VM to run a VM Maker image
fn main() {
    let builder = match std::env::consts::OS {
        "linux" => LinuxBuilder::default().boxed(),
        "macos" => MacBuilder::default().boxed(),
        "windows" => WindowsBuilder::default().boxed(),
        _ => {
            panic!("The platform you're compiling for is not supported");
        }
    };

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
