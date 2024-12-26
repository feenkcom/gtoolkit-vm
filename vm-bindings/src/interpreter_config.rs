use crate::parameters::InterpreterParameters;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InterpreterConfiguration {
    image: PathBuf,
    interactive_session: bool,
    should_print_stack_on_signals: bool,
    should_avoid_searching_segments_with_pinned_objects: bool,
    worker_thread: bool,
    arguments: Vec<String>,
}

impl InterpreterConfiguration {
    pub fn new(image: impl Into<PathBuf>) -> Self {
        Self {
            image: image.into(),
            interactive_session: false,
            should_print_stack_on_signals: false,
            should_avoid_searching_segments_with_pinned_objects: false,
            worker_thread: false,
            arguments: vec![],
        }
    }

    pub fn image(&self) -> &Path {
        self.image.as_path()
    }

    pub fn is_worker_thread(&self) -> bool {
        self.worker_thread
    }

    pub fn should_print_stack_on_signals(&self) -> bool {
        self.should_print_stack_on_signals
    }

    pub fn set_is_worker_thread(&mut self, worker_thread: bool) -> &mut Self {
        self.worker_thread = worker_thread;
        self
    }

    pub fn set_should_avoid_searching_segments_with_pinned_objects(
        &mut self,
        should_avoid_searching_segments_with_pinned_objects: bool,
    ) -> &mut Self {
        self.should_avoid_searching_segments_with_pinned_objects =
            should_avoid_searching_segments_with_pinned_objects;
        self
    }

    pub fn set_interactive_session(&mut self, interactive_session: bool) -> &mut Self {
        self.interactive_session = interactive_session;
        self
    }

    pub fn set_should_print_stack_on_signals(
        &mut self,
        should_print_stack_on_signals: bool,
    ) -> &mut Self {
        self.should_print_stack_on_signals = should_print_stack_on_signals;
        self
    }

    pub fn set_extra_arguments(&mut self, arguments: Vec<String>) -> &mut Self {
        self.arguments = arguments;
        self
    }

    pub fn create_interpreter_parameters(&self) -> InterpreterParameters {
        // Emscripten's MEMFS doesn't ave /proc/self/exe, therefore current_exe fails.
        let executable_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/main"));

        let mut vm_args: Vec<String> = vec![];
        vm_args.push(executable_path.as_os_str().to_str().unwrap().to_owned());
        vm_args.push(self.image.as_os_str().to_str().unwrap().to_owned());
        vm_args.extend(self.arguments.clone());

        let mut parameters = InterpreterParameters::from_args(vm_args);
        parameters.set_image_file_name(self.image.as_os_str().to_str().unwrap().to_owned());
        parameters.set_is_interactive_session(self.interactive_session);
        parameters.set_should_avoid_searching_segments_with_pinned_objects(
            self.should_avoid_searching_segments_with_pinned_objects,
        );
        parameters.set_is_worker(self.worker_thread);

        parameters
    }
}
