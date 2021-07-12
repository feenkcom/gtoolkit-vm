use crate::options::BundleOptions;
use std::path::{Path, PathBuf};

pub mod linux;
pub mod mac;
pub mod windows;

pub use crate::libraries::Library;
use crate::ExecutableOptions;
use std::process::Command;

pub trait Bundler {
    fn pre_compile(&self, _options: &ExecutableOptions) {}
    fn post_compile(&self, _options: &ExecutableOptions) {}

    fn compile_binary(&self, options: &ExecutableOptions) {
        std::env::set_var("CARGO_TARGET_DIR", options.target_dir());

        if let Some(vmmaker_vm) = options.vmmaker_vm() {
            std::env::set_var("VM_CLIENT_VMMAKER", vmmaker_vm);
        }

        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--package")
            .arg("vm-client")
            .arg("--bin")
            .arg(options.compiled_executable_name())
            .arg("--target")
            .arg(options.target().to_string());

        match options.verbose() {
            0 => {}
            1 => {
                command.arg("-v");
            }
            _ => {
                command.arg("-vv");
            }
        }

        if options.release() {
            command.arg("--release");
        }

        if !command.status().unwrap().success() {
            panic!("Failed to compile a vm-client")
        }
    }

    fn bundle(&self, options: &BundleOptions);

    fn ensure_third_party_requirements(&self, options: &BundleOptions) {
        options
            .libraries()
            .iter()
            .for_each(|library| library.ensure_requirements());
    }

    fn compile_third_party_libraries(&self, final_options: &BundleOptions) {
        let compiled_libraries_directory = self.compiled_libraries_directory(&final_options);

        if !compiled_libraries_directory.exists() {
            std::fs::create_dir_all(&compiled_libraries_directory).expect(&format!(
                "Could not create {}",
                compiled_libraries_directory.display()
            ));
        }

        final_options.libraries().iter().for_each(|library| {
            library
                .ensure_sources(&final_options)
                .expect(&format!("Could not validate sources of {}", library.name()));
            library.compile(&final_options);
            assert!(
                library.is_compiled(&final_options),
                "Compiled library must exist at {:?}",
                library.compiled_library(final_options).display()
            );

            let library_path = compiled_libraries_directory
                .join(library.compiled_library_name().file_name(library.name()));

            std::fs::copy(library.compiled_library(&final_options), &library_path).expect(
                &format!(
                    "Could not copy {} to {}",
                    library.compiled_library(&final_options).display(),
                    &library_path.display(),
                ),
            );
        })
    }

    fn bundle_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.bundle_location()
    }

    fn compilation_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.compilation_location()
    }

    fn default_bundle_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.default_bundle_location()
    }

    fn compiled_libraries_directory(&self, configuration: &BundleOptions) -> PathBuf {
        self.compilation_location(configuration)
            .join(Path::new("shared_libraries"))
    }

    fn compiled_libraries(&self, configuration: &BundleOptions) -> Vec<PathBuf> {
        self.compiled_libraries_directory(configuration)
            .read_dir()
            .unwrap()
            .map(|each| each.unwrap().path())
            .collect()
    }
}
