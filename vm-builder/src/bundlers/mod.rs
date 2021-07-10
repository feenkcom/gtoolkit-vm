use crate::options::FinalOptions;
use std::path::{Path, PathBuf};

pub mod linux;
pub mod mac;
pub mod windows;

pub use crate::libraries::Library;

pub trait Bundler {
    fn pre_compile(&self, _configuration: &FinalOptions) {}
    fn bundle(&self, configuration: &FinalOptions);
    fn post_compile(&self, _configuration: &FinalOptions) {}

    fn ensure_third_party_requirements(&self, final_options: &FinalOptions) {
        final_options
            .third_party_libraries()
            .iter()
            .for_each(|library| library.ensure_requirements());
    }

    fn compile_third_party_libraries(&self, final_options: &FinalOptions) {
        final_options
            .third_party_libraries()
            .iter()
            .for_each(|library| {
                library
                    .ensure_sources(&final_options)
                    .expect(&format!("Could not validate sources of {}", library.name()));
                library.compile(&final_options);
                assert!(
                    library.is_compiled(&final_options),
                    "Compiled library must exist at {:?}",
                    library.compiled_library(final_options).display()
                );

                let library_path = self.compiled_libraries_directory(&final_options).join(
                    library
                        .compiled_library(&final_options)
                        .file_name()
                        .unwrap(),
                );

                std::fs::copy(library.compiled_library(&final_options), &library_path).expect(
                    &format!(
                        "Could not copy {} to {}",
                        library.compiled_library(&final_options).display(),
                        &library_path.display(),
                    ),
                );
            })
    }

    fn bundle_location(&self, configuration: &FinalOptions) -> PathBuf {
        configuration.bundle_location()
    }

    fn compilation_location(&self, configuration: &FinalOptions) -> PathBuf {
        configuration.compilation_location()
    }

    fn default_bundle_location(&self, configuration: &FinalOptions) -> PathBuf {
        configuration.default_bundle_location()
    }

    fn app_name(&self, configuration: &FinalOptions) -> String {
        configuration.app_name()
    }

    fn compiled_executable_path(&self, configuration: &FinalOptions) -> PathBuf {
        let mut path_buf = self.compilation_location(configuration);
        match self.executable_extension(configuration) {
            None => {
                path_buf.push("vm_client");
            }
            Some(extension) => {
                path_buf.push(format!("{}.{}", "vm_client", extension));
            }
        }
        path_buf
    }

    fn compiled_cli_executable_path(&self, configuration: &FinalOptions) -> PathBuf {
        let mut path_buf = self.compilation_location(configuration);
        match self.executable_extension(configuration) {
            None => {
                path_buf.push("vm_client-cli");
            }
            Some(extension) => {
                path_buf.push(format!("{}.{}", "vm_client-cli", extension));
            }
        }
        path_buf
    }

    fn executable_name(&self, configuration: &FinalOptions) -> String {
        configuration.executable_name()
    }

    fn executable_extension(&self, configuration: &FinalOptions) -> Option<String> {
        configuration.executable_extension()
    }

    fn compiled_libraries_directory(&self, configuration: &FinalOptions) -> PathBuf {
        self.compilation_location(configuration)
            .join(Path::new("shared_libraries"))
    }

    fn compiled_libraries(&self, configuration: &FinalOptions) -> Vec<PathBuf> {
        self.compiled_libraries_directory(configuration)
            .read_dir()
            .unwrap()
            .map(|each| each.unwrap().path())
            .collect()
    }

    fn bundle_version(&self, configuration: &FinalOptions) -> String {
        let major = configuration.major_version();

        let minor = configuration.minor_version();
        let patch = configuration.patch_version();

        format!("{}.{}.{}", major, minor, patch)
    }

    fn bundle_identifier(&self, configuration: &FinalOptions) -> String {
        configuration
            .identifier()
            .as_ref()
            .map_or_else(|| self.app_name(configuration), |id| id.to_owned())
    }
}
