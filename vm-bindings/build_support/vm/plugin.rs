use crate::{Builder, BuilderTarget, CompilationUnit, Core, Unit, Dependency};
use cc::Build;
use file_matcher::FilesNamed;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Plugin {
    plugin: Unit,
    core: Core,
}

impl Plugin {
    pub fn new(name: impl Into<String>, core: &Core) -> Self {
        let mut unit = Unit::new(name, core.builder());
        unit.dependency(Dependency::Core(core.clone()));

        Self {
            plugin: unit,
            core: core.clone()
        }
    }

    pub fn extracted(name: impl Into<String>, core: &Core) -> Self {
        let name = name.into();
        let mut plugin = Self::new(name.clone(), core);

        let extracted_dir = core
            .builder()
            .vm_sources_directory()
            .join("extracted")
            .join("plugins")
            .join(name.clone());

        plugin.add_sources(
            FilesNamed::wildmatch("*.c")
                .within(extracted_dir.join("src/common"))
                .find()
                .unwrap(),
        );

        match core.builder().target() {
            BuilderTarget::MacOS => {
                plugin.add_sources(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/osx"))
                        .find()
                        .unwrap(),
                );

                // If MacOS specific version does not exist we add unix for MacOS
                if !extracted_dir.join("src/osx").exists() {
                    plugin.add_sources(
                        FilesNamed::wildmatch("*.c")
                            .within(extracted_dir.join("src/unix"))
                            .find()
                            .unwrap(),
                    );
                }
            }
            BuilderTarget::Linux => {
                plugin.add_sources(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/unix"))
                        .find()
                        .unwrap(),
                );
            }
            BuilderTarget::Windows => {
                plugin.add_sources(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/win"))
                        .find()
                        .unwrap(),
                );
            }
        }

        plugin.with_default_includes();

        plugin
    }

    pub fn get_includes(&self) -> &Vec<PathBuf> {
        self.plugin.get_includes()
    }

    pub fn with_default_includes(&mut self) {
        let extracted_dir = self
            .core
            .builder()
            .vm_sources_directory()
            .join("extracted")
            .join("plugins")
            .join(self.plugin.name());

        self.add_include(extracted_dir.join("include/common"));
        self.add_include(extracted_dir.join("src/common"));

        match self.builder().target() {
            BuilderTarget::MacOS => {
                self.add_include(extracted_dir.join("include/osx"));
                self.add_include(extracted_dir.join("src/osx"));
                if !extracted_dir.join("include/osx").exists() {
                    self.add_include(extracted_dir.join("include/unix"));
                }
                if !extracted_dir.join("src/osx").exists() {
                    self.add_include(extracted_dir.join("src/unix"));
                }
            }
            BuilderTarget::Linux => {
                self.add_include(extracted_dir.join("include/unix"));
                self.add_include(extracted_dir.join("src/unix"));
            }
            BuilderTarget::Windows => {
                self.add_include(extracted_dir.join("include/win"));
                self.add_include(extracted_dir.join("src/win"));
            }
        }
    }

    pub fn compile(&self) -> Build {
        let mut compilation_unit = self.plugin.clone();
        compilation_unit.add_includes(self.core.get_includes());

        for define in self.core.get_defines() {
            compilation_unit.define(&define.0, define.1.as_ref().map(|value| value.as_str()));
        }

        compilation_unit.flags(self.core.get_flags());

        let build = compilation_unit.compile();
        build
    }
}

impl CompilationUnit for Plugin {
    fn name(&self) -> &str {
        self.plugin.name()
    }

    fn builder(&self) -> Rc<dyn Builder> {
        self.plugin.builder()
    }

    fn add_include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.plugin.add_include(dir);
        self
    }

    fn add_source<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.plugin.add_source(dir);
        self
    }

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self {
        self.plugin.define(var, val);
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.plugin.flag(flag);
        self
    }

    fn dependency(&mut self, dependency: Dependency) -> &mut Self {
        self.plugin.dependency(dependency);
        self
    }
}
