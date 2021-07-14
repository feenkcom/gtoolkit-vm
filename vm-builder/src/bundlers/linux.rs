use crate::bundlers::Bundler;
use crate::options::BundleOptions;
use std::fs;
use std::process::Command;

pub struct LinuxBundler {}

impl LinuxBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for LinuxBundler {
    fn bundle(&self, options: &BundleOptions) {
        let bundle_location = options.bundle_location();
        let app_name = options.app_name();

        let app_dir = bundle_location.join(&app_name);
        let binary_dir = app_dir.join("bin");

        let library_dir_name = "lib";

        let library_dir = app_dir.join(library_dir_name);

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&binary_dir).unwrap();

        options.executables().iter().for_each(|executable| {
            let compiled_executable_path = options.compiled_executable_path(executable);
            let bundled_executable_path =
                binary_dir.join(options.bundled_executable_name(executable));
            match fs::copy(&compiled_executable_path, &bundled_executable_path) {
                Ok(_) => {
                    Command::new("patchelf")
                        .arg("--set-rpath")
                        .arg(format!("$ORIGIN/../{}", library_dir_name))
                        .status()
                        .unwrap();
                }
                Err(error) => {
                    panic!(
                        "Could not copy {} to {} due to {}",
                        &compiled_executable_path.display(),
                        &bundled_executable_path.display(),
                        error
                    );
                }
            };
        });

        fs_extra::copy_items(
            &self.compiled_libraries(options),
            &library_dir,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();
    }
}
