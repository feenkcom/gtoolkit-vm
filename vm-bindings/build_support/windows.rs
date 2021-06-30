use crate::build_support::Builder;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use titlecase::titlecase;

#[derive(Default, Clone)]
pub struct WindowsBuilder {}

impl Debug for WindowsBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl WindowsBuilder {
    fn pthreads_directory(&self) -> PathBuf {
        self.output_directory().join("pthreads")
    }

    fn libgit2_directory(&self) -> PathBuf {
        self.output_directory().join("libgit2-src")
    }

    fn libgit2_build_directory(&self) -> PathBuf {
        self.output_directory().join("libgit2-build")
    }

    fn sdl2_directory(&self) -> PathBuf {
        self.output_directory().join("sdl2-src")
    }

    fn sdl2_build_directory(&self) -> PathBuf {
        self.output_directory().join("sdl2-build")
    }

    fn libssh2_directory_name(&self) -> String {
        "libssh2-src".to_string()
    }

    fn libssh2_directory(&self) -> PathBuf {
        self.output_directory().join(self.libssh2_directory_name())
    }

    fn clone_pthread(&self) {
        if self.pthreads_directory().exists() {
            return;
        }

        Command::new("git")
            .current_dir(self.output_directory())
            .arg("clone")
            .arg("https://github.com/BrianGladman/pthreads.git")
            .status()
            .unwrap();
    }

    fn clone_libgit2(&self) {
        if self.libgit2_directory().exists() {
            return;
        }

        Command::new("git")
            .current_dir(self.output_directory())
            .arg("clone")
            .arg("-b")
            .arg("v1.1.0")
            .arg("https://github.com/libgit2/libgit2.git")
            .arg(self.libgit2_directory())
            .status()
            .unwrap();
    }

    fn clone_libssh2(&self) {
        if self.libssh2_directory().exists() {
            return;
        }

        Command::new("git")
            .current_dir(self.output_directory())
            .arg("clone")
            .arg("-b")
            .arg("libssh2-1.9.0")
            .arg("https://github.com/libssh2/libssh2.git")
            .arg(self.libssh2_directory())
            .status()
            .unwrap();
    }

    fn clone_sdl2(&self) {
        if self.sdl2_directory().exists() {
            return;
        }

        Command::new("git")
            .current_dir(self.output_directory())
            .arg("clone")
            .arg("-b")
            .arg("release-2.0.14")
            .arg("https://github.com/libsdl-org/SDL.git")
            .arg(self.sdl2_directory())
            .status()
            .unwrap();
    }

    fn compile_pthread(&self) {
        let solution = self
            .pthreads_directory()
            .join("build.vs")
            .join("pthreads.sln");

        assert!(
            self.pthreads_directory().exists(),
            "Pthread source folder must exist: {:?}",
            self.pthreads_directory().display()
        );
        assert!(
            solution.exists(),
            "Solution file must exist: {:?}",
            &solution.display()
        );

        Command::new("MSBuild")
            .current_dir(self.pthreads_directory())
            .arg(&solution)
            .arg("/p:Platform=x64")
            .arg(format!("/property:Configuration={}", self.profile()))
            .status()
            .unwrap();
    }

    fn compile_libgit2(&self) {
        if self.libgit2_binary().exists() {
            return;
        }

        Command::new("cmake")
            .current_dir(self.output_directory())
            .arg(self.cmake_build_type())
            .arg(format!(
                "-DEMBED_SSH_PATH=../../{}",
                self.libssh2_directory_name()
            ))
            .arg("-DBUILD_CLAR=OFF")
            .arg("-S")
            .arg(self.libgit2_directory())
            .arg("-B")
            .arg(self.libgit2_build_directory())
            .arg("-G")
            .arg("Visual Studio 16 2019")
            .arg("-A")
            .arg("x64")
            .status()
            .unwrap();

        Command::new("cmake")
            .current_dir(self.output_directory())
            .arg("--build")
            .arg(self.libgit2_build_directory())
            .arg("--config")
            .arg(self.profile())
            .status()
            .unwrap();
    }

    fn compile_sdl2(&self) {
        if self.sdl2_binary().exists() {
            return;
        }

        Command::new("cmake")
            .current_dir(self.output_directory())
            .arg(self.cmake_build_type())
            .arg("-S")
            .arg(self.sdl2_directory())
            .arg("-B")
            .arg(self.sdl2_build_directory())
            .arg("-G")
            .arg("Visual Studio 16 2019")
            .arg("-A")
            .arg("x64")
            .status()
            .unwrap();

        Command::new("cmake")
            .current_dir(self.output_directory())
            .arg("--build")
            .arg(self.sdl2_build_directory())
            .arg("--config")
            .arg(self.profile())
            .status()
            .unwrap();
    }

    fn libgit2_binary(&self) -> PathBuf {
        self.libgit2_build_directory()
            .join(self.profile())
            .join("git2.dll")
    }

    fn sdl2_binary(&self) -> PathBuf {
        self.sdl2_build_directory()
            .join(self.profile())
            .join(if self.is_debug() {
                "SDL2d.dll"
            } else {
                "SDL2.dll"
            })
    }

    fn pthreads_library_directory(&self) -> PathBuf {
        self.pthreads_directory()
            .join("lib")
            .join("x64")
            .join(self.profile())
    }

    fn export_dll_from_directory(
        &self,
        directory: &PathBuf,
        libraries: &mut Vec<(PathBuf, Option<String>)>,
    ) {
        directory
            .read_dir()
            .unwrap()
            .map(|each_entry| each_entry.unwrap())
            .map(|each_entry| each_entry.path())
            .filter(|each_path| each_path.is_file())
            .filter(|each_file| each_file.extension().is_some())
            .filter(|each_file| each_file.extension().unwrap().to_str().unwrap() == "dll")
            .for_each(|each| {
                if !self.includes_dll(&each, libraries) {
                    libraries.push((each, None))
                }
            });
    }

    fn includes_dll(&self, file: &PathBuf, libraries: &mut Vec<(PathBuf, Option<String>)>) -> bool {
        libraries
            .iter()
            .find(|each| each.0.file_name().unwrap() == file.file_name().unwrap())
            .is_some()
    }
}

impl Builder for WindowsBuilder {
    fn ensure_build_tools(&self) {
        which::which("pkg-config").expect("Could not find pkg-config. Please add it to PATH");
        which::which("cmake").expect("Could not find cmake. Please add it to PATH");
        which::which("git").expect("Could not find git. Please add it to PATH");
        which::which("MSBuild").expect("Could not find MSBuild. Please add it to PATH");
        which::which("clang").expect("Could not find clang. Please add it to PATH");
        which::which("clang++").expect("Could not find clang++. Please add it to PATH");
        which::which("lld").expect("Could not find lld. Please add it to PATH");
        if !Path::new(&std::env::var("LIBCLANG_PATH").expect("LIBCLANG_PATH must be set")).exists()
        {
            panic!(
                "LIBCLANG_PATH must exist: {:?}",
                &std::env::var("LIBCLANG_PATH")
            )
        }
        if !Path::new(&std::env::var("LLVM_HOME").expect("LLVM_HOME must be set")).exists()
        {
            panic!(
                "LLVM_HOME must exist: {:?}",
                &std::env::var("LLVM_HOME")
            )
        }
    }

    fn vm_binary(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("vm")
            .join("PharoVMCore.dll")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("vm")
    }

    fn generate_sources(&self) {
        self.clone_pthread();
        //self.clone_libgit2();
        //self.clone_libssh2();
        //self.clone_sdl2();
        self.compile_pthread();
        //self.compile_libgit2();
        //self.compile_sdl2();

        std::fs::create_dir_all(self.compiled_libraries_directory()).unwrap();

        cmake::Config::new(self.vm_sources_directory())
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_PTHREADW32", "ON")
            .define("PTHREADW32_DIR", self.pthreads_library_directory())
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF")
            .generator("Visual Studio 16 2019")
            .generator_toolset("ClangCL")
            .build();
    }

    fn compile_sources(&self) {}

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("win")
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");

        println!(
            "cargo:rustc-link-search={}\\build\\vm",
            self.output_directory()
                .join("build")
                .join(titlecase(&self.profile()))
                .display()
        );
    }

    fn shared_libraries_to_export(&self) -> Vec<(PathBuf, Option<String>)> {
        let mut libraries = vec![];

        self.export_dll_from_directory(
            &self
                .output_directory()
                .join("build")
                .join("build")
                .join("vm"),
            &mut libraries,
        );

        self.export_dll_from_directory(
            &self
                .output_directory()
                .join("build")
                .join(titlecase(&self.profile())),
            &mut libraries,
        );

        //libraries.push((self.libgit2_binary(), Some("libgit2.dll".to_string())));
        //libraries.push((self.sdl2_binary(), Some("SDL2.dll".to_string())));

        libraries
    }
}
