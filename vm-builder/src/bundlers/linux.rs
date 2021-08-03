use crate::bundlers::Bundler;
use crate::options::BundleOptions;
use crate::{Executable, ExecutableOptions};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub struct LinuxBundler {}

impl LinuxBundler {
    pub fn new() -> Self {
        Self {}
    }

    fn library_dir_name(&self) -> &str {
        "lib"
    }

    fn set_rpath(&self, binary: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        which::which("patchelf")?;

        let binary = binary.as_ref();
        if !Command::new("patchelf")
            .arg("--set-rpath")
            .arg(format!("$ORIGIN/../{}/", self.library_dir_name()))
            .arg(binary)
            .status()?
            .success()
        {
            return Err(Box::new(UserFacingError::new(format!(
                "Failed to set RUNPATH of {}",
                binary.display(),
            ))));
        }
        Ok(())
    }
}

impl Bundler for LinuxBundler {
    fn bundle(&self, options: &BundleOptions) {
        let bundle_location = options.bundle_location();
        let app_name = options.app_name();

        let app_dir = bundle_location.join(&app_name);
        let binary_dir = app_dir.join("bin");

        let library_dir = app_dir.join(self.library_dir_name());

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&binary_dir).unwrap();
        fs::create_dir(&library_dir).unwrap();

        options.executables().iter().for_each(|executable| {
            let compiled_executable_path = options.compiled_executable_path(executable);
            let bundled_executable_path =
                binary_dir.join(options.bundled_executable_name(executable));
            fs::copy(&compiled_executable_path, &bundled_executable_path)
                .map_err(|error| {
                    UserFacingError::new(format!(
                        "Could not copy {} to {} due to {}",
                        &compiled_executable_path.display(),
                        &bundled_executable_path.display(),
                        error
                    ));
                })
                .expect("Failed to copy");
        });

        self.compiled_libraries(options)
            .iter()
            .for_each(|compiled_library_path| {
                let bundled_library_path =
                    library_dir.join(compiled_library_path.file_name().unwrap());

                match fs::copy(&compiled_library_path, &bundled_library_path) {
                    Ok(_) => {
                        self.set_rpath(&bundled_library_path).unwrap();
                    }
                    Err(error) => {
                        panic!(
                            "Could not copy {} to {} due to {}",
                            &compiled_library_path.display(),
                            &bundled_library_path.display(),
                            error
                        );
                    }
                };
            });
    }
    fn post_compile(
        &self,
        bundle_options: &BundleOptions,
        executable: &Executable,
        _executable_options: &ExecutableOptions,
    ) {
        self.set_rpath(bundle_options.compiled_executable_path(executable))
            .expect("Failed to set rpath");
    }

    fn clone_bundler(&self) -> Box<dyn Bundler> {
        Box::new(Clone::clone(self))
    }
}
