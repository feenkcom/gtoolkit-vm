use crate::build_support::vm_unit::CompilationUnit;
use crate::{Builder, Unit};
use cc::Build;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Core {
    core: Unit,
}

impl Core {
    pub fn new(name: impl Into<String>, builder: Rc<dyn Builder>) -> Self {
        Self {
            core: Unit::new(name, builder),
        }
    }

    pub fn get_includes(&self) -> &Vec<PathBuf> {
        self.core.get_includes()
    }

    pub fn get_defines(&self) -> &Vec<(String, Option<String>)> {
        self.core.get_defines()
    }

    pub fn get_flags(&self) -> &Vec<String> {
        self.core.get_flags()
    }

    pub fn check_include_files(
        &mut self,
        header_name: impl AsRef<str>,
        define: impl AsRef<str>,
    ) -> &mut Self {
        let has_header = bindgen::Builder::default()
            .header_contents(
                header_name.as_ref(),
                &format!("#include <{}>", header_name.as_ref()),
            )
            .generate()
            .is_ok();

        if has_header {
            self.define(define.as_ref(), None);
            println!("{} - found", header_name.as_ref());
        } else {
            println!("{} - not found", header_name.as_ref());
        }
        self
    }

    pub fn compile(&self) -> Build {
        let build = self.core.compile();

        let compiler = build.get_compiler();
        let mut command = compiler.to_command();
        command.current_dir(self.core.output_directory());
        command
            .arg("-Wl,-all_load")
            .arg(format!("lib{}.a", self.name()))
            .arg("-l")
            .arg("ffi")
            .arg("-framework")
            .arg("AppKit")
            .arg("-o")
            .arg(format!("lib{}.dylib", self.name()));

        println!("{:?}", &command);

        if !command.status().unwrap().success() {
            panic!("Failed to create a dylib");
        };

        std::fs::copy(
            self.core.output_directory().join(self.binary_name()),
            self.core.artefact_directory().join(self.binary_name()),
        )
        .unwrap();

        build
    }
}

impl CompilationUnit for Core {
    fn name(&self) -> &str {
        self.core.name()
    }

    fn builder(&self) -> Rc<dyn Builder> {
        self.core.builder()
    }

    fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.core.include(dir);
        self
    }

    fn file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.core.file(dir);
        self
    }

    fn remove_file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.core.remove_file(dir);
        self
    }

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self {
        self.core.define(var, val);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.core.flag(flag);
        self
    }
}
