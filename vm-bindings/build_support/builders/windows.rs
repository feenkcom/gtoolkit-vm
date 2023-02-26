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
        match which::which("vcpkg") {
            Ok(vcpkg) => vcpkg,
            Err(_) => {
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
            }
        }
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

    fn pthreads_install_directory() -> PathBuf {
        Self::out_dir().join("pthreads")
    }

    fn pthreads_directory() -> PathBuf {
        Self::pthreads_install_directory().join(Self::vcpkg_triplet())
    }

    pub fn pthreads_name() -> &'static str {
        "pthreads"
    }

    pub fn pthreads_lib() -> PathBuf {
        Self::pthreads_directory().join("lib")
    }

    pub fn pthreads_lib_name() -> &'static str {
        "pthreadVC3"
    }

    fn install_pthreads() -> PathBuf {
        let vcpkg = Self::prepare_vcpkg();
        let pthread_install_directory = Self::pthreads_install_directory();
        let triplet = Self::vcpkg_triplet();

        let pthreads_directory = Self::pthreads_directory();
        if !pthreads_directory.exists() {
            let output = Command::new(&vcpkg)
                .current_dir(&Self::out_dir())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&["--triplet", triplet])
                .args(&["install", "pthreads"])
                .arg("--x-install-root")
                .arg(&pthread_install_directory)
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

        pthreads_directory
    }

    pub fn ffi_name() -> &'static str {
        "libffi"
    }

    fn ffi_install_directory() -> PathBuf {
        Self::out_dir().join(Self::ffi_name())
    }

    fn ffi_directory() -> PathBuf {
        Self::ffi_install_directory().join(Self::vcpkg_triplet())
    }

    pub fn install_ffi() -> PathBuf {
        let vcpkg = Self::prepare_vcpkg();
        let ffi_install_directory = Self::ffi_install_directory();
        let triplet = Self::vcpkg_triplet();

        let ffi_directory = Self::ffi_directory();
        if !ffi_directory.exists() {
            let output = Command::new(&vcpkg)
                .current_dir(&Self::out_dir())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&["--triplet", triplet])
                .args(&["install", Self::ffi_name()])
                .arg("--x-install-root")
                .arg(&ffi_install_directory)
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
