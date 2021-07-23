use crate::build_support::Builder;
use file_matcher::{FileNamed, OneEntry};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
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

    fn pthreads_library_directory(&self) -> PathBuf {
        self.pthreads_directory()
            .join("lib")
            .join("x64")
            .join(self.profile())
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
        if !Path::new(&std::env::var("LLVM_HOME").expect("LLVM_HOME must be set")).exists() {
            panic!("LLVM_HOME must exist: {:?}", &std::env::var("LLVM_HOME"))
        }
    }

    fn vm_binary(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join(titlecase(&self.profile()))
            .join("PharoVMCore.dll")
    }

    fn compiled_libraries_directory(&self) -> PathBuf {
        self.output_directory()
            .join("build")
            .join("build")
            .join("vm")
    }

    fn compile_sources(&self) {
        self.clone_pthread();
        self.compile_pthread();

        std::fs::create_dir_all(self.compiled_libraries_directory()).unwrap();

        let mut config = cmake::Config::new(self.vm_sources_directory());
        config
            .define("COMPILE_EXECUTABLE", "OFF")
            .define("FEATURE_LIB_PTHREADW32", "ON")
            .define("PTHREADW32_DIR", self.pthreads_library_directory())
            .define("FEATURE_LIB_GIT2", "OFF")
            .define("FEATURE_LIB_SDL2", "OFF")
            .generator("Visual Studio 16 2019")
            .generator_toolset("ClangCL");

        if let Some(vm_maker) = self.vm_maker() {
            let path: PathBuf = vm_maker;
            let mut path = path.into_os_string();
            #[cfg(windows)]
            {
                // CMake doesn't like unescaped `\`s paths
                use std::ffi::OsString;
                use std::os::windows::ffi::{OsStrExt, OsStringExt};
                let wchars = path
                    .encode_wide()
                    .map(|wchar| {
                        if wchar == b'\\' as u16 {
                            '/' as u16
                        } else {
                            wchar
                        }
                    })
                    .collect::<Vec<_>>();
                path = OsString::from_wide(&wchars);
            }
            config.define("GENERATE_PHARO_VM", path);
        }

        config.build();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("win")
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");

        println!(
            "cargo:rustc-link-search={}",
            self.output_directory()
                .join("build")
                .join(titlecase(&self.profile()))
                .display()
        );
    }

    fn shared_libraries_to_export(&self) -> Vec<OneEntry> {
        let vm_build = &self
            .output_directory()
            .join("build")
            .join(titlecase(&self.profile()));

        let ffi_test = &self
            .output_directory()
            .join("build")
            .join("build")
            .join("ffiTestLibrary")
            .join(titlecase(&self.profile()));

        let third_party_build = self
            .output_directory()
            .join("build")
            .join("build")
            .join("vm");

        let mut libraries = vec![];

        vec![
            // core
            FileNamed::exact("PharoVMCore.dll"),
            // plugins
            FileNamed::exact("B2DPlugin.dll"),
            FileNamed::exact("BitBltPlugin.dll"),
            FileNamed::exact("DSAPrims.dll"),
            FileNamed::exact("FileAttributesPlugin.dll"),
            FileNamed::exact("FilePlugin.dll"),
            FileNamed::exact("JPEGReaderPlugin.dll"),
            FileNamed::exact("JPEGReadWriter2Plugin.dll"),
            FileNamed::exact("LargeIntegers.dll"),
            FileNamed::exact("LocalePlugin.dll"),
            FileNamed::exact("MiscPrimitivePlugin.dll"),
            FileNamed::exact("SocketPlugin.dll"),
            FileNamed::exact("SqueakSSL.dll"),
            FileNamed::exact("SurfacePlugin.dll"),
            FileNamed::exact("UUIDPlugin.dll"),
        ]
        .into_iter()
        .map(|library| library.within(&vm_build))
        .for_each(|each| libraries.push(each));

        vec![
            // third party
            FileNamed::exact("ffi.dll"),
            FileNamed::exact("libbz2-1.dll"),
            FileNamed::exact("libcairo-2.dll"),
            FileNamed::exact("libexpat-1.dll"),
            FileNamed::exact("libfontconfig-1.dll"),
            FileNamed::exact("libpixman-1-0.dll"),
            FileNamed::exact("libpng16-16.dll"),
        ]
        .into_iter()
        .map(|library| library.within(&third_party_build))
        .for_each(|each| libraries.push(each));

        libraries.push(FileNamed::exact("TestLibrary.dll").within(&ffi_test));

        libraries
    }

    fn boxed(self) -> Box<dyn Builder> {
        Box::new(self)
    }
}
