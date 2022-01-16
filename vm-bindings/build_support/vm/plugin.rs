use crate::{Builder, BuilderTarget, CompilationUnit, Core, Unit};
use cc::Build;
use file_matcher::FilesNamed;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Dependency {
    Core(Core),
    Plugin(Plugin),
    Framework(String),
    Library(String),
}

#[derive(Debug, Clone)]
pub struct Plugin {
    plugin: Unit,
    core: Core,
    dependencies: Vec<Dependency>,
}

impl Plugin {
    pub fn new(name: impl Into<String>, core: &Core) -> Self {
        let mut dependencies = Vec::new();
        dependencies.push(Dependency::Core(core.clone()));

        Self {
            plugin: Unit::new(name, core.builder()),
            core: core.clone(),
            dependencies,
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

    pub fn add_dependency(&mut self, dependency: Dependency) -> &mut Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn compile(&self) -> Build {
        let mut compilation_unit = self.plugin.clone();
        compilation_unit.add_includes(self.core.get_includes());

        for define in self.core.get_defines() {
            compilation_unit.define(&define.0, define.1.as_ref().map(|value| value.as_str()));
        }

        compilation_unit.flags(self.core.get_flags());

        let build = compilation_unit.compile();

        let compiler = build.get_compiler();
        let mut command = compiler.to_command();
        command.current_dir(self.plugin.output_directory());
        command
            .arg("-Wl,-all_load")
            .arg(format!("lib{}.a", self.plugin.name()));

        for dependency in &self.dependencies {
            match dependency {
                Dependency::Core(core) => {
                    command.arg("-l").arg(core.name());
                }
                Dependency::Plugin(plugin) => {
                    command.arg("-l").arg(plugin.name());
                }
                Dependency::Framework(framework) => {
                    command.arg("-framework").arg(framework);
                }
                Dependency::Library(library) => {
                    command.arg("-l").arg(library);
                }
            }
        }

        command.arg("-L").arg(".").arg("-o").arg(self.binary_name());

        if !command.status().unwrap().success() {
            panic!("Failed to create a dylib");
        };

        std::fs::copy(
            self.plugin.output_directory().join(self.binary_name()),
            self.plugin.artefact_directory().join(self.binary_name()),
        )
        .unwrap();

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
}
