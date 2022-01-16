use crate::{Builder, CompilationUnit, Feature, Unit};
use cc::Build;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Core {
    core: Unit,
    features: Vec<Feature>,
}

impl Core {
    pub fn new(name: impl Into<String>, builder: Rc<dyn Builder>) -> Self {
        Self {
            core: Unit::new(name, builder),
            features: vec![],
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

    pub fn add_feature(&mut self, feature: Feature) -> &mut Self {
        self.features.push(feature);
        self
    }

    pub fn compile(&self) -> Build {
        let mut core_with_features = self.core.clone();
        for feature in &self.features {
            core_with_features = core_with_features.merge(feature.get_unit());
        }
        let build = core_with_features.compile();

        let compiler = build.get_compiler();
        let mut command = compiler.to_command();
        command.current_dir(self.output_directory());
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
            self.output_directory().join(self.binary_name()),
            self.artefact_directory().join(self.binary_name()),
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

    fn add_include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.core.add_include(dir);
        self
    }

    fn add_source<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.core.add_source(dir);
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
