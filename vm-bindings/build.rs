extern crate bindgen;
extern crate fs_extra;

use std::path::{Path, PathBuf};
use std::process::Command;

use fs_extra::{copy_items, dir};

use std::fs::ReadDir;
use std::io::Error;
use std::{env, fs};

fn run<F>(name: &str, mut configure: F, panic: bool)
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    if !configured.status().unwrap().success() {
        if panic {
            panic!("failed to execute {:?}", configured);
        } else {
            println!("failed to execute {:?}", configured);
        }
    }
}

const VM_PATH: &str = "../opensmalltalk-vm";
const PATCHES_PATH: &str = "patch";

fn patch_opensmalltalk_vm() {
    let paths = match fs::read_dir(PATCHES_PATH) {
        Ok(paths) => paths,
        Err(_) => return,
    };

    for path in paths {
        let patch_name = path.unwrap().file_name();
        let patch_file_name = format!(
            "../{}/{}",
            PATCHES_PATH,
            patch_name.clone().into_string().unwrap()
        );

        println!("Patching {}", &patch_file_name);
        run(
            "git",
            |options| {
                options
                    .current_dir(VM_PATH)
                    .arg("apply")
                    .arg("--whitespace=fix")
                    .arg(patch_file_name.clone())
            },
            false,
        );
    }
}

fn compile_opensmalltalk_vm() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let vm_binary = format!("{}/build/dist/Pharo.app/Contents/MacOS/Pharo", &out_dir);
    if Path::new(&vm_binary).exists() {
        return;
    }

    run(
        "cmake",
        |options| {
            options
                .current_dir(VM_PATH)
                .arg("-S")
                .arg(VM_PATH)
                .arg("-B")
                .arg(out_dir.as_str())
        },
        true,
    );

    run(
        "make",
        |options| options.current_dir(out_dir.as_str()).arg("install"),
        true,
    );
}

fn generate_bindings() {
    let build_dir = env::var("OUT_DIR").unwrap();

    // Tell cargo to tell rustc to link the shared library.
    println!("cargo:rustc-link-lib=PharoVMCore");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!(
        "cargo:rustc-link-search=target/{}/Plugins",
        std::env::var("PROFILE").unwrap()
    );
    println!(
        "cargo:rustc-link-search={}/build/dist/Pharo.app/Contents/MacOS/Plugins",
        &build_dir
    );

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!(
            "{}/build/dist/include/pharovm/pharoClient.h",
            &build_dir
        ))
        .header(format!(
            "{}/build/dist/include/pharovm/sqMemoryAccess.h",
            &build_dir
        ))
        .clang_arg(format!("-I{}/build/dist/include", &build_dir))
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
    let build_dir = env::var("OUT_DIR").unwrap();

    let compiled_plugins_dir =
        format!("{}/build/dist/Pharo.app/Contents/MacOS/Plugins", &build_dir);

    let export_plugins_folder = PathBuf::new()
        .join("..")
        .join(std::env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string()))
        .join(std::env::var("TARGET").unwrap())
        .join(std::env::var("PROFILE").unwrap());

    println!(
        "Current working dir: {}",
        std::env::current_dir().unwrap().to_str().unwrap()
    );
    println!(
        "Compiled plugins dir ({}): {}",
        Path::new(&compiled_plugins_dir).exists(),
        &compiled_plugins_dir
    );
    println!(
        "Export plugins dir ({}): {}",
        Path::new(&export_plugins_folder).exists(),
        &export_plugins_folder.display()
    );

    let mut options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
    options.overwrite = true;

    let mut from_paths = Vec::new();
    from_paths.push(&compiled_plugins_dir);

    copy_items(&from_paths, &export_plugins_folder, &options).unwrap();

    fs::copy(
        format!("{}/build/dist/libSDL2-2.0d.dylib", &build_dir),
        format!("{}/Plugins/libSDL2-2.0.0.dylib", &export_plugins_folder.display()),
    )
    .unwrap();
}

fn main() {
    //patch_opensmalltalk_vm();
    compile_opensmalltalk_vm();
    generate_bindings();
    package_libraries();
}
