use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;
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

impl BuilderTarget {
    pub fn is_unix(&self) -> bool {
        match self {
            BuilderTarget::MacOS | BuilderTarget::Linux => true,
            BuilderTarget::Windows => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        match self {
            BuilderTarget::MacOS | BuilderTarget::Linux => false,
            BuilderTarget::Windows => true,
        }
    }
}

pub fn compile_ffi(builder: Rc<dyn Builder>) -> anyhow::Result<()> {
    let ffi_sources = builder.output_directory().join("libffi");

    if !ffi_sources.exists() {
        Command::new("git")
            .current_dir(builder.output_directory())
            .arg("clone")
            .arg("https://github.com/syrel/libffi.git")
            .status()
            .unwrap();

        // checkout the version of libffi that works
        Command::new("git")
            .current_dir(&ffi_sources)
            .arg("checkout")
            .arg("af975d04e64dfc2116078160b3b75524eb6bf241")
            .status()
            .unwrap();
    }

    let ffi_build = builder.output_directory().join("libffi-build");
    if !ffi_build.exists() {
        std::fs::create_dir_all(&ffi_build)?;
    }
    cmake::Config::new(ffi_sources)
        .static_crt(true)
        .out_dir(&ffi_build)
        .build();

    let ffi_binary_name = match builder.target() {
        BuilderTarget::MacOS => "libffi.dylib",
        BuilderTarget::Linux => "libffi.so",
        BuilderTarget::Windows => "ffi.dll",
    };

    std::fs::copy(
        ffi_build.join("bin").join(ffi_binary_name),
        builder.artefact_directory().join(ffi_binary_name),
    )?;

    Ok(())
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
