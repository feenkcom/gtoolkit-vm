use file_matcher::{OneEntry, OneEntryCopier};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{env, fmt, fs};

const VM_CLIENT_VMMAKER_VM_VAR: &str = "VM_CLIENT_VMMAKER";
const VM_CLIENT_VMMAKER_IMAGE_VAR: &str = "VM_CLIENT_VMMAKER_IMAGE";

pub trait Builder: Debug {
    fn is_compiled(&self) -> bool {
        self.vm_binary().exists()
    }

    fn profile(&self) -> String {
        std::env::var("PROFILE").unwrap()
    }

    fn is_debug(&self) -> bool {
        self.profile() == "debug"
    }

    fn ensure_build_tools(&self) {}

    fn vmmaker_vm(&self) -> Option<PathBuf> {
        std::env::var(VM_CLIENT_VMMAKER_VM_VAR).map_or(None, |path| {
            let path = Path::new(&path);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                panic!(
                    "Specified {} does not exist: {}",
                    VM_CLIENT_VMMAKER_VM_VAR,
                    path.display()
                );
            }
        })
    }

    fn vmmaker_image(&self) -> Option<PathBuf> {
        std::env::var(VM_CLIENT_VMMAKER_IMAGE_VAR).map_or(None, |path| {
            let path = Path::new(&path);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                panic!(
                    "Specified {} does not exist: {}",
                    VM_CLIENT_VMMAKER_IMAGE_VAR,
                    path.display()
                );
            }
        })
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
        let target = std::env::var("CARGO_TARGET");
        let mut path = PathBuf::new()
            .join("..")
            .join(std::env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string()));

        if let Ok(target) = target {
            path = path.join(target);
        }

        path.join(self.profile()).join("shared_libraries")
    }

    fn compile_sources(&self);

    fn squeak_include_directory(&self) -> PathBuf {
        self.vm_sources_directory()
            .join("extracted")
            .join("vm")
            .join("include")
    }

    fn common_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("common")
    }

    fn platform_include_directory(&self) -> PathBuf;

    fn generated_config_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("include")
            .join("pharovm")
    }

    fn generated_include_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("generated")
            .join("64")
            .join("vm")
            .join("include")
    }

    fn generate_bindings(&self) {
        let include_dir = self.vm_sources_directory().join("include");

        let generated_vm_include_dir = self.generated_include_directory();
        assert!(
            generated_vm_include_dir.exists(),
            "Generated vm include directory must exist: {:?}",
            generated_vm_include_dir.display()
        );

        let generated_config_directory = self.generated_config_directory();
        assert!(
            generated_config_directory.exists(),
            "Generated config.h directory must exist: {:?}",
            generated_config_directory.display()
        );

        let bindings = bindgen::Builder::default()
            .whitelist_function("vm_.*")
            .whitelist_function("free")
            .header(
                include_dir
                    .join("pharovm")
                    .join("pharoClient.h")
                    .display()
                    .to_string(),
            )
            .clang_arg(format!("-I{}", &include_dir.display()))
            .clang_arg(format!("-I{}", &include_dir.join("pharovm").display()))
            .clang_arg(format!("-I{}", generated_config_directory.display()))
            .clang_arg(format!("-I{}", generated_vm_include_dir.display()))
            .clang_arg(format!("-I{}", self.common_include_directory().display()))
            .clang_arg(format!("-I{}", self.platform_include_directory().display()))
            .clang_arg("-DLSB_FIRST=1")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(self.output_directory().join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    fn link_libraries(&self);

    fn export_shared_libraries(&self) {
        if !self.exported_libraries_directory().exists() {
            fs::create_dir_all(self.exported_libraries_directory()).unwrap();
        }

        for shared_library in self.shared_libraries_to_export() {
            let target = self.exported_libraries_directory();
            match shared_library.copy(&target) {
                Ok(_) => {}
                Err(error) => {
                    panic!(
                        "Could not copy {:?} to {} due to {}",
                        &shared_library,
                        &target.display(),
                        error
                    )
                }
            }
        }
    }

    fn shared_libraries_to_export(&self) -> Vec<OneEntry>;

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

    fn boxed(self) -> Box<dyn Builder>;
}
