use crate::BuildOptions;
use std::path::{Path, PathBuf};

pub mod mac;

pub trait Bundler {
    fn bundle(&self, configuration: &BuildOptions);

    fn bundle_location(&self, configuration: &BuildOptions) -> PathBuf {
        configuration.bundle_dir.as_ref().map_or_else(
            || self.default_bundle_location(configuration),
            |bundle_dir| PathBuf::new().join(Path::new(&bundle_dir)),
        )
    }

    fn compilation_location(&self, configuration: &BuildOptions) -> PathBuf {
        let mut bundle_dir = PathBuf::new();
        bundle_dir.push(configuration.target_dir.as_ref().unwrap());
        bundle_dir.push(configuration.target.as_ref().unwrap().to_string());
        bundle_dir.push(if configuration.release {
            "release"
        } else {
            "debug"
        });
        bundle_dir
    }

    fn default_bundle_location(&self, configuration: &BuildOptions) -> PathBuf {
        let mut path_buf = self.compilation_location(configuration);
        path_buf.push("bundle");
        path_buf
    }

    fn app_name(&self, configuration: &BuildOptions) -> String {
        configuration
            .app_name
            .as_ref()
            .map_or("VM".to_owned(), |name| name.to_owned())
    }

    fn compiled_executable_path(&self, configuration: &BuildOptions) -> PathBuf {
        let mut path_buf = self.compilation_location(configuration);
        path_buf.push("vm_client");
        path_buf
    }

    fn executable_name(&self, configuration: &BuildOptions) -> String {
        configuration
            .executable_name
            .as_ref()
            .map_or_else(|| self.app_name(configuration), |name| name.to_owned())
    }

    fn compiled_libraries_directory(&self, configuration: &BuildOptions) -> PathBuf {
        self.compilation_location(configuration)
            .join(Path::new("shared_libraries"))
    }

    fn compiled_libraries(&self, configuration: &BuildOptions) -> Vec<PathBuf> {
        self.compiled_libraries_directory(configuration)
            .read_dir()
            .unwrap()
            .map(|each| each.unwrap().path())
            .collect()
    }

    fn bundle_version(&self, configuration: &BuildOptions) -> String {
        let major = configuration.major_version.unwrap_or_else(|| {
            if configuration.minor_version.is_some() | configuration.patch_version.is_some() {
                0
            } else {
                1
            }
        });

        let minor = configuration.minor_version.unwrap_or(0);
        let patch = configuration.patch_version.unwrap_or(0);

        format!("{}.{}.{}", major, minor, patch)
    }

    fn bundle_identifier(&self, configuration: &BuildOptions) -> String {
        configuration
            .identifier
            .as_ref()
            .map_or_else(|| "".to_owned(), |id| id.to_owned())
    }
}
