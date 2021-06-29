use regex::Regex;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, fs};

pub(crate) enum Name<'a> {
    Exact(&'a str),
    Any(Vec<&'a str>),
    Regex(&'a str),
    Optional(&'a str),
}

impl<'a> Name<'a> {
    pub(crate) fn find_file(&self, directory: &PathBuf) -> Option<PathBuf> {
        match self {
            Name::Exact(name) => {
                let file = directory.join(name);
                assert!(file.exists(), "File named {} must exist!", name);
                Some(file)
            }
            Name::Any(names) => {
                let files = names
                    .iter()
                    .map(|each| directory.join(each))
                    .filter(|each| each.exists())
                    .collect::<Vec<PathBuf>>();
                assert!(
                    files.len() > 0,
                    "At least one file out of {:?} must exist",
                    names
                );
                Some(files.first().unwrap().clone())
            }
            Name::Regex(regex) => {
                let file = self.find_file_in_directory_matching(regex, directory);
                assert!(file.is_some(), "At least one file must match {}", regex);
                file
            }
            Name::Optional(name) => {
                let file = directory.join(name);
                if file.exists() {
                    Some(file)
                } else {
                    None
                }
            }
        }
    }

    fn find_file_in_directory_matching(
        &self,
        file_name_regex: &str,
        directory: &PathBuf,
    ) -> Option<PathBuf> {
        directory.read_dir().map_or(None, |dir| {
            dir.filter(|each_entry| each_entry.is_ok())
                .map(|each_entry| each_entry.unwrap())
                .map(|each_entry| each_entry.path())
                .filter(|each_path| each_path.is_file())
                .filter(|each_path| {
                    each_path.file_name().map_or(false, |file_name| {
                        file_name.to_str().map_or(false, |file_name| {
                            Regex::new(file_name_regex)
                                .map_or(false, |regex| regex.is_match(file_name))
                        })
                    })
                })
                .collect::<Vec<PathBuf>>()
                .first()
                .map(|path| path.clone())
        })
    }
}

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

    fn should_embed_debug_symbols(&self) -> bool {
        std::env::var("VM_CLIENT_EMBED_DEBUG_SYMBOLS").map_or(false, |value| value == "true")
    }

    fn cmake_build_type(&self) -> String {
        (if self.is_debug() {
            "-DCMAKE_BUILD_TYPE=RelWithDebInfo"
        } else if self.should_embed_debug_symbols() {
            "-DCMAKE_BUILD_TYPE=RelWithDebInfo"
        } else {
            "-DCMAKE_BUILD_TYPE=Release"
        })
        .to_string()
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
            .join(self.profile())
            .join("shared_libraries")
    }

    fn generate_sources(&self);

    fn compile_sources(&self) {
        Command::new("cmake")
            .arg("--build")
            .arg(self.output_directory())
            .arg("--config")
            .arg(self.profile())
            .status()
            .unwrap();
    }

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
    fn generated_config_directory(&self) -> PathBuf;
    fn generated_include_directory(&self) -> PathBuf {
        self.output_directory()
            .join("generated")
            .join("64")
            .join("vm")
            .join("include")
    }

    fn generate_bindings(&self) {
        let include_dir = self.vm_sources_directory().join("include");

        let generated_vm_include_dir = self.generated_include_directory();

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
            .clang_arg(format!(
                "-I{}",
                &self.generated_config_directory().display()
            ))
            .clang_arg(format!("-I{}", &generated_vm_include_dir.display()))
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
            let origin = shared_library.0.clone();

            let target_file_name = shared_library.1.as_ref().map_or_else(
                || origin.file_name().unwrap().to_str().unwrap().to_string(),
                |name| name.to_string(),
            );

            let target = self.exported_libraries_directory().join(target_file_name);
            match fs::copy(&origin, &target) {
                Ok(_) => {}
                Err(error) => {
                    panic!(
                        "Could not copy {} to {} due to {}",
                        &origin.display(),
                        &target.display(),
                        error
                    )
                }
            }
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
