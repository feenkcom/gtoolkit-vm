use std::ffi::OsString;
use std::path::{Path, PathBuf};
use crate::parameters::InterpreterParameters;

#[derive(Debug, Clone)]
pub struct InterpreterConfiguration {
    image: PathBuf,
    interactive_session: bool,
    handle_errors: bool,
    worker_thread: bool,
    arguments: Vec<String>,
}

impl InterpreterConfiguration {
    pub fn new(image: impl Into<PathBuf>) -> Self {
        Self {
            image: image.into(),
            interactive_session: false,
            handle_errors: true,
            worker_thread: false,
            arguments: vec![]
        }
    }

    pub fn image(&self) -> &Path {
        self.image.as_path()
    }

    pub fn is_worker_thread(&self) -> bool {
        self.worker_thread
    }

    pub fn should_handle_errors(&self) -> bool {
        self.handle_errors
    }

    pub fn set_is_worker_thread(&mut self, worker_thread: bool) -> &mut Self {
        self.worker_thread = worker_thread;
        self
    }

    pub fn set_interactive_session(&mut self, interactive_session: bool) -> &mut Self {
        self.interactive_session = interactive_session;
        self
    }

    pub fn set_should_handle_errors(&mut self, handle_errors: bool) -> &mut Self {
        self.handle_errors = handle_errors;
        self
    }

    pub fn set_extra_arguments(&mut self, arguments: Vec<String>) -> &mut Self {
        self.arguments = arguments;
        self
    }

    pub fn create_interpreter_parameters(&self) -> InterpreterParameters {
        let executable_path = std::env::current_exe().unwrap();

        let mut vm_args: Vec<String> = vec![];
        vm_args.push(executable_path.as_os_str().to_str().unwrap().to_owned());
        vm_args.push(self.image.as_os_str().to_str().unwrap().to_owned());
        vm_args.extend(self.arguments.clone());

        let mut parameters = InterpreterParameters::from_args(vm_args);
        parameters.set_image_file_name(self.image.as_os_str().to_str().unwrap().to_owned());
        parameters.set_is_interactive_session(self.interactive_session);

        parameters
    }
}