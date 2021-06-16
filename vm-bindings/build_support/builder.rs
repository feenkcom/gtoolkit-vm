use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, fs};

pub trait Builder: Debug {
    fn is_compiled(&self) -> bool {
        self.vm_binary().exists()
    }

    fn output_directory(&self) -> PathBuf {
        Path::new(env::var("OUT_DIR").unwrap().as_str()).to_path_buf()
    }

    /// Return a path to the compiled vm binary.
    /// For example, on Mac it would be an executable inside of the .app bundle
    fn vm_binary(&self) -> PathBuf;

    fn vm_sources_directory(&self) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
            .join("opensmalltalk-vm")
    }

    fn compiled_libraries_directory(&self) -> PathBuf;

    fn exported_libraries_directory(&self) -> PathBuf {
        PathBuf::new()
            .join("..")
            .join(std::env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string()))
            .join(std::env::var("TARGET").unwrap())
            .join(std::env::var("PROFILE").unwrap())
            .join("shared_libraries")
    }

    fn generate_sources(&self) {
        Command::new("cmake")
            .arg("-S")
            .arg(self.vm_sources_directory())
            .arg("-B")
            .arg(self.output_directory())
            .status()
            .unwrap();
    }

    fn compile_sources(&self) {
        Command::new("make")
            .current_dir(self.output_directory())
            .arg("install")
            .status()
            .unwrap();
    }

    fn generate_bindings(&self) {
        let include_dir = self
            .output_directory()
            .join("build")
            .join("dist")
            .join("include");

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header(
                include_dir
                    .join("pharovm")
                    .join("pharoClient.h")
                    .display()
                    .to_string(),
            )
            .header(
                include_dir
                    .join("pharovm")
                    .join("sqMemoryAccess.h")
                    .display()
                    .to_string(),
            )
            .clang_arg(format!("-I{}", &include_dir.display()))
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

    fn link_libraries(&self);

    fn export_shared_libraries(&self) {
        if !self.exported_libraries_directory().exists() {
            fs::create_dir_all(self.exported_libraries_directory()).unwrap();
        }

        for shared_library in self.shared_libraries_to_export() {
            let origin = shared_library.0.clone();

            let target_file_name = shared_library.1.as_ref().map_or_else(
                || origin.file_name().unwrap().to_str().unwrap().to_string(),
                |name| name.to_string(),
            );

            let target = self.exported_libraries_directory().join(target_file_name);
            fs::copy(origin, target).unwrap();
        }
    }

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)>;

    fn print_directories(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entry(&"is_compiled".to_string(), &self.is_compiled())
            .entry(
                &"output_directory".to_string(),
                &self.output_directory().display(),
            )
            .entry(&"vm_binary".to_string(), &self.vm_binary().display())
            .entry(
                &"vm_sources_directory".to_string(),
                &self.vm_sources_directory().display(),
            )
            .entry(
                &"compiled_libraries_directory".to_string(),
                &self.compiled_libraries_directory().display(),
            )
            .entry(
                &"exported_libraries_directory".to_string(),
                &self.exported_libraries_directory().display(),
            )
            .finish()
    }
}
