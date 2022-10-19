use std::{env, fmt};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const SOURCES_DIRECTORY: &str = "pharo-vm";

#[derive(Debug, Clone, PartialEq)]
pub enum BuilderTarget {
    MacOS,
    Linux,
    Windows,
}

impl BuilderTarget {
    pub fn is_unix(&self) -> bool {
        match self {
            BuilderTarget::MacOS | BuilderTarget::Linux => true,
            BuilderTarget::Windows => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        self == &BuilderTarget::Windows
    }
    pub fn is_macos(&self) -> bool {
        self == &BuilderTarget::MacOS
    }
}

pub trait Builder: Debug {
    fn target(&self) -> BuilderTarget;

    fn profile(&self) -> String {
        env::var("PROFILE").unwrap()
    }

    fn is_debug(&self) -> bool {
        self.profile() == "debug"
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
            .join(SOURCES_DIRECTORY)
    }

    fn prepare_environment(&self);

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

        let extra_headers = env::current_dir().unwrap().join("extra");

        let mut builder = bindgen::Builder::default();
        builder = builder
            .allowlist_function("vm_.*")
            .allowlist_function("free")
            .allowlist_function("calloc")
            .allowlist_function("malloc")
            .allowlist_function("memcpy")
            .allowlist_function("registerCurrentThreadToHandleExceptions")
            .allowlist_function("installErrorHandlers")
            .allowlist_function("setProcessArguments")
            .allowlist_function("setProcessEnvironmentVector")
            .allowlist_function("getVMExports")
            .allowlist_function("setVMExports")
            // re-export the internal methods
            .allowlist_function("exportSqGetInterpreterProxy")
            .allowlist_function("exportOsCogStackPageHeadroom")
            .allowlist_function("exportGetHandler")
            .allowlist_function("exportReadAddress")
            .allowlist_function("exportInstantiateClassIsPinned")
            .allowlist_function("exportFirstBytePointerOfDataObject")
            .allowlist_function("exportStatFullGCUsecs")
            .allowlist_function("exportStatScavengeGCUsecs")
            .allowlist_function("setVmRunOnWorkerThread")
            .allowlist_function("setLogger")
            .allowlist_function("setShouldLog")
            .allowlist_type("sqInt")
            .allowlist_type("usqInt")
            .allowlist_type("sqExport")
            .allowlist_type("VirtualMachine")
            .header(
                include_dir
                    .join("pharovm")
                    .join("pharoClient.h")
                    .display()
                    .to_string(),
            )
            .header(extra_headers.join("sqExport.h").display().to_string())
            .header(extra_headers.join("exported.h").display().to_string())
            .header(extra_headers.join("setLogger.h").display().to_string())
            .clang_arg(format!("-I{}", &include_dir.display()))
            .clang_arg(format!("-I{}", &include_dir.join("pharovm").display()))
            .clang_arg(format!("-I{}", generated_config_directory.display()))
            .clang_arg(format!("-I{}", generated_vm_include_dir.display()))
            .clang_arg(format!("-I{}", self.common_include_directory().display()))
            .clang_arg(format!("-I{}", self.platform_include_directory().display()))
            .clang_arg("-DLSB_FIRST=1")
            // the following allows bindgen to parse headers when building for arm64 Windows
            .clang_arg("-D_NO_CRT_STDIO_INLINE")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks));

        let bindings = builder
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(self.output_directory().join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");
        println!(
            "cargo:rustc-link-search={}",
            self.artefact_directory().display()
        );
    }

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
            .finish()
    }

    fn boxed(self) -> Rc<dyn Builder>;
}
