use crate::bundlers::Bundler;
use crate::BuildOptions;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env::args;

pub struct LinuxBundler {}

impl LinuxBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for LinuxBundler {
    fn bundle(&self, configuration: &BuildOptions) {
        let bundle_location = self.bundle_location(configuration);
        let app_name = self.app_name(configuration);

        let app_dir = bundle_location.join(&app_name);
        let binary_dir = app_dir.join("bin");
        let library_dir = app_dir.join("lib");

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&binary_dir).unwrap();
        fs::create_dir(&library_dir).unwrap();

        let target_executable_path = binary_dir.join(&self.executable_name(configuration));

        match fs::copy(
            self.compiled_executable_path(configuration),
            &target_executable_path,
        ) {
            Ok(_) => {}
            Err(error) => {
                panic!(
                    "Could not copy {} to {} due to {}",
                    self.compiled_executable_path(configuration).display(),
                    &target_executable_path.display(),
                    error
                );
            }
        };

        fs_extra::copy_items(
            &self.compiled_libraries(configuration),
            library_dir,
            &fs_extra::dir::CopyOptions::new(),
        ).unwrap();

        Command::new("patchelf")
            .arg("--set-rpath")
            .arg("$ORIGIN/../lib")
            .arg(&target_executable_path)
            .status()
            .unwrap();
    }
}
