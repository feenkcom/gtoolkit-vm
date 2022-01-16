use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fmt};

const VM_CLIENT_VMMAKER_VM_VAR: &str = "VM_CLIENT_VMMAKER";
const VM_CLIENT_VMMAKER_IMAGE_VAR: &str = "VM_CLIENT_VMMAKER_IMAGE";

#[derive(Debug, Clone)]
pub enum BuilderTarget {
    MacOS,
    Linux,
    Windows,
}

pub trait Builder: Debug {
    fn target(&self) -> BuilderTarget;

    fn profile(&self) -> String {
        std::env::var("PROFILE").unwrap()
    }

    fn is_debug(&self) -> bool {
        self.profile() == "debug"
    }

    fn ensure_build_tools(&self) {}

    /// Return a list of all generated sources
    fn generated_sources(&self) -> Vec<PathBuf> {
        let root = self
            .output_directory()
            .join("generated")
            .join("64")
            .join("vm")
            .join("src");

        [
            root.join("cogit.c"),
            #[cfg(not(feature = "gnuisation"))]
            root.join("cointerp.c"),
            #[cfg(feature = "gnuisation")]
            root.join("gcc3x-cointerp.c"),
        ]
        .to_vec()
    }

    /// Return a list of extracted sources shared among all platforms
    fn common_extracted_sources(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    /// Return a list of platform specific extracted sources specific for this platform
    fn platform_extracted_sources(&self) -> Vec<PathBuf>;

    /// Return a list of all extracted sources including common ones and platform specific
    fn extracted_sources(&self) -> Vec<PathBuf> {
        let mut sources = Vec::new();
        sources.append(&mut self.common_extracted_sources());
        sources.append(&mut self.platform_extracted_sources());
        sources
    }

    /// Return a list of support sources
    fn support_sources(&self) -> Vec<PathBuf> {
        let root = self.vm_sources_directory();
        [
            root.join("src/debug.c"),
            root.join("src/utils.c"),
            root.join("src/errorCode.c"),
            root.join("src/nullDisplay.c"),
            root.join("src/externalPrimitives.c"),
            root.join("src/client.c"),
            root.join("src/pathUtilities.c"),
            root.join("src/parameterVector.c"),
            root.join("src/parameters.c"),
            root.join("src/fileDialogCommon.c"),
            root.join("src/stringUtilities.c"),
            root.join("src/imageAccess.c"),
            root.join("src/semaphores/platformSemaphore.c"),
            root.join("extracted/vm/src/common/heartbeat.c"),
        ]
        .to_vec()
    }

    /// Return a list of all sources to compile
    fn sources(&self) -> Vec<PathBuf> {
        let mut sources = Vec::new();
        sources.append(&mut self.support_sources());
        sources.append(&mut self.generated_sources());
        sources.append(&mut self.extracted_sources());
        sources
    }

    fn platform_includes(&self) -> Vec<PathBuf>;

    fn includes(&self) -> Vec<PathBuf> {
        let mut includes = Vec::new();
        includes.append(&mut self.platform_includes());
        includes.push(self.vm_sources_directory().join("include"));
        includes.push(self.output_directory().join("generated/64/vm/include"));
        includes
    }

    fn output_directory(&self) -> PathBuf {
        Path::new(env::var("OUT_DIR").unwrap().as_str()).to_path_buf()
    }

    fn artefact_directory(&self) -> PathBuf {
        let dir = self.output_directory();
        dir.parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

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
        self.generated_include_directory()
    }

    fn generated_include_directory(&self) -> PathBuf {
        self.output_directory()
            .join("generated")
            .join("64")
            .join("vm")
            .join("include")
    }

    fn generate_bindings(&self) {
        // Rerun the build script of CMakeLists file changes
        println!(
            "cargo:rerun-if-changed={:?}",
            self.vm_sources_directory().join("CMakeLists.txt").display()
        );

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

    fn print_directories(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entry(
                &"output_directory".to_string(),
                &self.output_directory().display(),
            )
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

    fn boxed(self) -> Rc<dyn Builder>;
}
