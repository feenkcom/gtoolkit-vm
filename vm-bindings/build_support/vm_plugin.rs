use crate::build_support::vm_unit::CompilationUnit;
use crate::{Builder, BuilderTarget, Core, Unit};
use cc::Build;
use file_matcher::FilesNamed;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Dependency {
    Core(Core),
    Plugin(Plugin),
    Framework(String),
}

#[derive(Debug, Clone)]
pub struct Plugin {
    plugin: Unit,
    core: Core,
    dependencies: Vec<Dependency>,
}

impl Plugin {
    pub fn new(name: impl Into<String>, core: Core) -> Self {
        let mut dependencies = Vec::new();
        dependencies.push(Dependency::Core(core.clone()));

        Self {
            plugin: Unit::new(name, core.builder()),
            core,
            dependencies,
        }
    }

    pub fn extracted(name: impl Into<String>, core: Core) -> Self {
        let name = name.into();
        let mut plugin = Self::new(name.clone(), core.clone());

        let extracted_dir = core.builder()
            .vm_sources_directory()
            .join("extracted")
            .join("plugins")
            .join(name.clone());

        plugin.files(
            FilesNamed::wildmatch("*.c")
                .within(extracted_dir.join("src/common"))
                .find()
                .unwrap(),
        );

        match core.builder().target() {
            BuilderTarget::MacOS => {
                plugin.files(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/osx"))
                        .find()
                        .unwrap(),
                );
                plugin.files(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/unix"))
                        .find()
                        .unwrap(),
                );
            }
            BuilderTarget::Linux => {
                plugin.files(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/unix"))
                        .find()
                        .unwrap(),
                );
            }
            BuilderTarget::Windows => {
                plugin.files(
                    FilesNamed::wildmatch("*.c")
                        .within(extracted_dir.join("src/win"))
                        .find()
                        .unwrap(),
                );
            }
        }

        plugin.add_includes();

        plugin
    }

    pub fn sources(&mut self, sources: impl AsRef<str>) -> &mut Self {
        let sources = sources.as_ref();

        let template = Template::new(sources);
        let mut data = HashMap::<String, String>::new();
        data.insert(
            "generated".to_string(),
            self.builder()
                .output_directory()
                .join("generated")
                .display()
                .to_string(),
        );
        data.insert(
            "sources".to_string(),
            self.builder().vm_sources_directory().display().to_string(),
        );
        let rendered = template.render_string(&data).unwrap();
        let path = PathBuf::from(rendered);
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let directory = path.parent().unwrap().to_path_buf();

        let files = FilesNamed::wildmatch(file_name)
            .within(directory)
            .find()
            .unwrap();
        self.files(files);
        self
    }

    pub fn add_includes(&mut self) {
        let extracted_dir = self
            .core
            .builder()
            .vm_sources_directory()
            .join("extracted")
            .join("plugins")
            .join(self.plugin.name());
        self.include(extracted_dir.join("include/common"));

        match self.builder().target() {
            BuilderTarget::MacOS => {
                self.include(extracted_dir.join("include/osx"));
                self.include(extracted_dir.join("include/unix"));
            }
            BuilderTarget::Linux => {
                self.include(extracted_dir.join("include/unix"));
            }
            BuilderTarget::Windows => {
                self.include(extracted_dir.join("include/win"));
            }
        }
    }

    pub fn add_dependency(&mut self, dependency: Dependency) -> &mut Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn compile(&self) -> Build {
        let mut compilation_unit = self.plugin.clone();
        compilation_unit.includes(self.core.get_includes());

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

    fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.plugin.include(dir);
        self
    }

    fn file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.plugin.file(dir);
        self
    }

    fn remove_file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.plugin.remove_file(dir);
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
