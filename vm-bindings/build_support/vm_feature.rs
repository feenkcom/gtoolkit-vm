use crate::{Builder, CompilationUnit, Core, Unit};
use std::path::Path;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Feature {
    feature: Unit,
}

impl Feature {
    pub fn new(name: impl Into<String>, core: &Core) -> Self {
        Self {
            feature: Unit::new(name, core.builder()),
        }
    }

    pub fn get_unit(&self) -> &Unit {
        &self.feature
    }
}

impl CompilationUnit for Feature {
    fn name(&self) -> &str {
        self.feature.name()
    }

    fn builder(&self) -> Rc<dyn Builder> {
        self.feature.builder()
    }

    fn add_include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.feature.add_include(dir);
        self
    }

    fn add_source<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.feature.add_source(dir);
        self
    }

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self {
        self.feature.define(var, val);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.feature.flag(flag);
        self
    }
}
