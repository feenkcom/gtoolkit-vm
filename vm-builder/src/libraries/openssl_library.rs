use crate::options::{BundleOptions, Target};
use crate::{Library, LibraryGitLocation, LibraryLocation, NativeLibrary};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct OpenSSLLibrary {
    location: LibraryLocation,
}

impl OpenSSLLibrary {
    pub fn new() -> Self {
        Self {
            location: LibraryLocation::Git(
                LibraryGitLocation::new("https://github.com/openssl/openssl.git")
                    .tag("OpenSSL_1_1_1k"),
            ),
        }
    }

    pub fn compiler(&self, options: &BundleOptions) -> &str {
        match options.target() {
            Target::X8664appleDarwin => "darwin64-x86_64-cc",
            Target::AArch64appleDarwin => "darwin64-arm64-cc",
            Target::X8664pcWindowsMsvc => "VC-WIN64A",
            Target::X8664UnknownlinuxGNU => "linux-x86_64-clang",
        }
    }
}

impl Library for OpenSSLLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "openssl"
    }

    fn force_compile(&self, options: &BundleOptions) {
        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir).expect(&format!("Could not create {:?}", &out_dir));
        }

        let makefile_dir = options.target_dir().join(self.name());

        let configure = Command::new("perl")
            .current_dir(&makefile_dir)
            .arg(self.source_directory(options).join("Configure"))
            .arg(format!("--{}", options.profile()))
            .arg(format!(
                "--prefix={}",
                self.native_library_prefix(options).display()
            ))
            .arg(format!(
                "--openssldir={}",
                self.native_library_prefix(options).display()
            ))
            .arg(self.compiler(options))
            .arg("OPT_LEVEL=3")
            .arg("no-shared")
            .status()
            .unwrap();

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }

        let make = Command::new("make")
            .current_dir(&makefile_dir)
            .arg("install_sw")
            .status()
            .unwrap();

        if !make.success() {
            panic!("Could not compile {}", self.name());
        }
    }

    fn compiled_library_directories(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        unimplemented!()
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool {
        false
    }

    fn ensure_requirements(&self, _options: &BundleOptions) {
        which::which("make").expect("Could not find `make`");
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl NativeLibrary for OpenSSLLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        options.target_dir().join(self.name()).join("build")
    }

    fn native_library_dependency_prefixes(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }

    fn clone_native_library(&self) -> Box<dyn NativeLibrary> {
        Box::new(Clone::clone(self))
    }
}
