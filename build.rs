extern crate bindgen;
extern crate fs_extra;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use fs_extra::dir::{copy, get_dir_content};
use fs_extra::{copy_items, dir};

use std::fs;

fn run<F>(name: &str, mut configure: F)
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    if !configured.status().unwrap().success() {
        panic!("failed to execute {:?}", configured);
    }
}

const VM_PATH: &str = "opensmalltalk-vm";
const PATCHES_PATH: &str = "patch";

fn compile_opensmalltalk_vm() {
    run("cmake", |options| options.current_dir(VM_PATH).arg("."));

    run("make", |options| {
        options.current_dir(VM_PATH).arg("install")
    });
}

fn generate_bindings() {
    let build_dir = format!("{}/build", VM_PATH);

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=PharoVMCore");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");

    println!(
        "cargo:rustc-link-search={}/Plugins",
        env::var("OUT_DIR").unwrap()
    );
    println!("cargo:rustc-link-search=target/release/Plugins");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/dist/include/pharovm/pharoClient.h", build_dir))
        .header(format!(
            "{}/dist/include/pharovm/sqMemoryAccess.h",
            build_dir
        ))
        .clang_arg(format!("-I{}/dist/include", build_dir))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the src/bindings.rs file.
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn package_libraries() {
    let mut options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
    options.overwrite = true;
    let mut from_paths = Vec::new();
    from_paths.push(format!(
        "{}/build/dist/Pharo.app/Contents/MacOS/Plugins",
        VM_PATH
    ));

    copy_items(&from_paths, "target/debug", &options);
    copy_items(&from_paths, "target/release", &options);
}

fn patch_opensmalltalk_vm() {
    let paths = fs::read_dir(PATCHES_PATH).unwrap();

    for path in paths {
        let patch_name = path.unwrap().file_name();
        let patch_file_name = format!("../{}/{}",PATCHES_PATH,patch_name.clone().into_string().unwrap());

        println!("Patching {}",&patch_file_name);
        run("git", |options| options.current_dir(VM_PATH).arg("apply").arg(patch_file_name.clone()));
    }
}

fn revert_opensmalltalk_vm() {
    let paths = fs::read_dir(PATCHES_PATH).unwrap();

    for path in paths {
        let patch_name = path.unwrap().file_name();
        let patch_file_name = format!("../{}/{}",PATCHES_PATH,patch_name.clone().into_string().unwrap());

        println!("Reverting {}",&patch_file_name);
        run("git", |options| options.current_dir(VM_PATH).arg("apply").arg("-R").arg(patch_file_name.clone()));
    }
}

fn main() {
    patch_opensmalltalk_vm();
    compile_opensmalltalk_vm();
    revert_opensmalltalk_vm();
    generate_bindings();
    package_libraries();
}
