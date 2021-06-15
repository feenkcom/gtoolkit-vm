extern crate vm_bindings;
extern crate nfd;

use vm_bindings::{VMParameters, VM};
use nfd::Response;

fn main() {
    let mut parameters = VMParameters::from_args();

    let result = nfd::dialog().filter("image").open().unwrap_or_else(|e| {
        panic!(e);
    });

    match result {
        Response::Okay(file_path) => {
            parameters.set_image_file_name(file_path);
        },
        Response::OkayMultiple(files) => {
            return;
        },
        Response::Cancel => {
            return;
        },
    }

    VM::start(parameters);
}
