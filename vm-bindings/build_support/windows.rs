use crate::{Builder, BuilderTarget};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct WindowsBuilder {}

impl Debug for WindowsBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_directories(f)
    }
}

impl WindowsBuilder {
    fn pthreads_directory(&self) -> PathBuf {
        self.output_directory().join("pthreads")
    }

    fn clone_pthread(&self) {
        if self.pthreads_directory().exists() {
            return;
        }

        Command::new("git")
            .current_dir(self.output_directory())
            .arg("clone")
            .arg("https://github.com/BrianGladman/pthreads.git")
            .status()
            .expect("Could not clone repository. Is git installed?");

        // checkout the version of pthreads that works
        Command::new("git")
            .current_dir(self.pthreads_directory())
            .arg("checkout")
            .arg("c49d9e1bce919638f46c82655a2117e9ccda4bb9")
            .status()
            .unwrap();
    }

    fn compile_pthread(&self) {
        let solution = self
            .pthreads_directory()
            .join("build.vs")
            .join("pthreads.sln");

        assert!(
            self.pthreads_directory().exists(),
            "Pthread source folder must exist: {:?}",
            self.pthreads_directory().display()
        );
        assert!(
            solution.exists(),
            "Solution file must exist: {:?}",
            &solution.display()
        );

        let mut msbuild =
            cc::windows_registry::find("msvc", "msbuild").expect("Could not find MSBuild.");
        msbuild
            .current_dir(self.pthreads_directory())
            .arg(&solution)
            .arg("/p:Platform=x64")
            .arg(format!("/property:Configuration={}", self.profile()))
            .status()
            .expect("Could not compile pthreads.");
    }
}

impl Builder for WindowsBuilder {
    fn target(&self) -> BuilderTarget {
        BuilderTarget::Windows
    }

    fn prepare_environment(&self) {
        self.clone_pthread();
        self.compile_pthread();
    }

    fn platform_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("win")
    }

    fn boxed(self) -> Rc<dyn Builder> {
        Rc::new(self)
    }
}
