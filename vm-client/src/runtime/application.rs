use std::path::{Path, PathBuf};

use vm_bindings::InterpreterConfiguration;

use crate::{
    executable_working_directory, pick_image_with_dialog, search_image_file_within_directories,
    AppOptions, ApplicationError, Constellation, Result,
};

#[derive(Debug, Clone)]
pub struct Application {
    working_directory: PathBuf,
    image: PathBuf,
}

impl Application {
    pub fn new(options: AppOptions) -> Result<Self> {
        let mut image = options.image().map(|image| image.to_path_buf());
        if image.is_none() {
            let possible_image_directories =
                vec![std::env::current_dir()?, executable_working_directory()?];
            image = search_image_file_within_directories(possible_image_directories);
        }
        if image.is_none() {
            let mut default_dialog_directory: Option<PathBuf> =
                Some(executable_working_directory()?);
            #[cfg(target_os = "macos")]
            {
                let translocation = crate::platform::mac::translocation::MacTranslocation::new()?;
                if translocation.is_translocated()? {
                    default_dialog_directory = Some(std::env::current_dir()?);
                }
            }
            image = pick_image_with_dialog(default_dialog_directory);
        }

        let image = image.ok_or_else(|| ApplicationError::ImageFileNotFound)?;

        let working_directory = image
            .parent()
            .ok_or_else(|| ApplicationError::NoParentDirectory(image.clone()))?
            .to_path_buf();

        Ok(Self {
            working_directory,
            image,
        })
    }

    pub fn start(&self) -> Result<()> {
        std::env::set_current_dir(self.working_directory.as_path())?;

        let mut configuration = InterpreterConfiguration::new(self.image.clone());
        configuration.set_interactive_session(true);
        configuration.set_should_handle_errors(true);
        configuration.set_is_worker_thread(false);

        Constellation::run(configuration);
        Ok(())
    }

    pub fn executable_name(&self) -> Result<String> {
        let executable_path = self.executable_path()?;
        let executable_name = executable_path
            .file_name()
            .ok_or_else(|| ApplicationError::FailedToGetFileName(executable_path.clone()))?;
        let executable_name = executable_name.to_str().ok_or_else(|| {
            ApplicationError::FailedToConvertOsString(executable_name.to_os_string())
        })?;
        Ok(executable_name.to_string())
    }

    pub fn executable_path(&self) -> Result<PathBuf> {
        let executable = std::env::args()
            .next()
            .ok_or_else(|| ApplicationError::NoExecutableInArguments)?;
        Ok(PathBuf::from(executable))
    }

    pub fn working_directory(&self) -> &Path {
        self.working_directory.as_path()
    }

    pub fn process_arguments(&self) -> Vec<String> {
        std::env::args().collect()
    }
}
