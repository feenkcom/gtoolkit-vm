use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

use platforms::Platform;

use crate::Builder;

#[derive(Clone, Debug)]
pub struct WindowsBuilder {
    platform: Platform,
}

impl WindowsBuilder {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    fn prepare_vcpkg() -> PathBuf {
        which::which("vcpkg").unwrap_or_else(|_| {
            let vcpkg_directory = Self::out_dir().join("vcpkg");

            if !vcpkg_directory.exists() {
                let status = Command::new("git")
                    .current_dir(Self::out_dir())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .arg("clone")
                    .arg("https://github.com/Microsoft/vcpkg.git")
                    .status()
                    .expect("Could not clone repository. Is git installed?");

                if !status.success() {
                    panic!("Could not clone vcpkg repository. Is git installed?")
                }
            }

            let vcpkg = vcpkg_directory.join("vcpkg.exe");

            if !vcpkg.exists() {
                let status = Command::new("cmd")
                    .current_dir(&vcpkg_directory)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(&["/C", ".\\bootstrap-vcpkg.bat"])
                    .status()
                    .expect("failed to execute process");

                if !status.success() {
                    panic!("Failed bootstrap vcpkg.")
                }
            }

            if !vcpkg.exists() {
                panic!(
                    "Could not find vcpkg executable in {}.",
                    &vcpkg_directory.display()
                );
            }

            vcpkg
        })
    }

    pub fn vcpkg_triplet() -> &'static str {
        let target = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        match target.as_str() {
            "x86_64" => "x64-windows-static",
            "aarch64" => "arm64-windows-static",
            _ => {
                panic!("Unsupported target: {}", &target)
            }
        }
    }

    fn out_dir() -> PathBuf {
        Path::new(env::var("OUT_DIR").unwrap().as_str()).to_path_buf()
    }

    fn vcpkg_root(vcpkg: &Path) -> PathBuf {
        if let Ok(root) = env::var("VCPKG_ROOT") {
            return PathBuf::from(root);
        }

        let vcpkg_parent = vcpkg
            .parent()
            .unwrap_or_else(|| panic!("Could not infer vcpkg root from {}", vcpkg.display()));

        for directory in vcpkg_parent.ancestors() {
            if directory.join(".vcpkg-root").exists()
                || (directory.join("scripts").is_dir() && directory.join("triplets").is_dir())
                || (directory.join("ports").is_dir() && directory.join("versions").is_dir())
            {
                return directory.to_path_buf();
            }
        }

        vcpkg_parent.to_path_buf()
    }

    fn vcpkg_packages_directory() -> PathBuf {
        let vcpkg = Self::prepare_vcpkg();
        Self::vcpkg_root(&vcpkg).join("packages")
    }

    fn pthreads_install_directory() -> PathBuf {
        Self::vcpkg_packages_directory()
    }

    fn pthreads_directory() -> PathBuf {
        Self::pthreads_install_directory().join(format!(
            "{}_{}",
            Self::pthreads_name(),
            Self::vcpkg_triplet()
        ))
    }

    pub fn pthreads_name() -> &'static str {
        "pthreads"
    }

    pub fn pthreads_lib() -> PathBuf {
        Self::pthreads_directory().join("lib")
    }

    pub fn pthreads_include() -> PathBuf {
        Self::pthreads_directory().join("include")
    }

    pub fn pthreads_lib_name() -> &'static str {
        "pthreadVC3"
    }

    fn install_pthreads() -> PathBuf {
        let vcpkg = Self::prepare_vcpkg();
        let triplet = Self::vcpkg_triplet();

        let pthreads_directory = Self::pthreads_directory();
        let pthreads_library =
            Self::pthreads_lib().join(format!("{}.lib", Self::pthreads_lib_name()));
        if !pthreads_library.exists() {
            let output = Command::new(&vcpkg)
                .current_dir(&Self::out_dir())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&["--triplet", triplet])
                .args(&["install", "pthreads"])
                .arg("--editable")
                .output()
                .expect("Failed to execute command to build pthreads");

            if !output.status.success() {
                println!("{}", String::from_utf8(output.stdout).unwrap());
                panic!(
                    "Failed to install pthreads: {}",
                    String::from_utf8(output.stderr).unwrap()
                )
            }
        }

        if !pthreads_library.exists() {
            panic!(
                "Could not find pthreads library at {} after vcpkg install.",
                pthreads_library.display()
            )
        }

        pthreads_directory
    }

    pub fn ffi_name() -> &'static str {
        "libffi"
    }

    fn ffi_install_directory() -> PathBuf {
        Self::vcpkg_packages_directory()
    }

    fn ffi_directory() -> PathBuf {
        Self::ffi_install_directory().join(format!(
            "{}_{}",
            Self::ffi_name(),
            Self::vcpkg_triplet()
        ))
    }

    pub fn ffi_include() -> PathBuf {
        Self::ffi_directory().join("include")
    }

    pub fn install_ffi() -> PathBuf {
        let vcpkg = Self::prepare_vcpkg();
        let triplet = Self::vcpkg_triplet();

        let ffi_directory = Self::ffi_directory();
        let ffi_lib = ffi_directory.join("lib");
        let ffi_library = ffi_lib.join(format!("{}.lib", Self::ffi_name()));
        if !ffi_library.exists() {
            let output = Command::new(&vcpkg)
                .current_dir(&Self::out_dir())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&["--triplet", triplet])
                .args(&["install", Self::ffi_name()])
                .output()
                .expect("Failed to execute command to build ffi");

            if !output.status.success() {
                println!("{}", String::from_utf8(output.stdout).unwrap());
                panic!(
                    "Failed to install ffi: {}",
                    String::from_utf8(output.stderr).unwrap()
                )
            }
        }

        if !ffi_library.exists() {
            match std::fs::read_dir(&ffi_lib) {
                Ok(entries) => {
                    println!("Contents of {}:", ffi_lib.display());
                    for entry in entries.flatten() {
                        println!("  {}", entry.path().display());
                    }
                }
                Err(error) => {
                    println!("Could not read {}: {}", ffi_lib.display(), error);
                }
            }
            panic!(
                "Could not find ffi library at {} after vcpkg install.",
                ffi_library.display()
            )
        }

        ffi_directory
    }
}

impl Builder for WindowsBuilder {
    fn platform(&self) -> &Platform {
        &self.platform
    }

    fn prepare_environment(&self) {
        let pthread_install = Self::install_pthreads();
        println!("Pthreads installed in {}", pthread_install.display());
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("win")
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
