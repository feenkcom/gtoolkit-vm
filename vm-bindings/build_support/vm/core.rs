use crate::build_support::Dependency;
use crate::{Builder, CompilationUnit, Feature, Unit};
use cc::Build;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone, Serialize)]
pub struct Core {
    #[serde(flatten)]
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

    pub fn get_includes(&self) -> &Vec<String> {
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

    fn add_include(&mut self, dir: impl AsRef<str>) -> &mut Self {
        self.core.add_include(dir);
        self
    }

    fn add_source(&mut self, dir: impl AsRef<str>) -> &mut Self {
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

    fn dependency(&mut self, dependency: Dependency) -> &mut Self {
        self.core.dependency(dependency);
        self
    }
}
