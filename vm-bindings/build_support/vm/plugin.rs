use std::rc::Rc;

use cc::Build;
use serde::Serialize;

use crate::{Builder, CompilationUnit, Core, Dependency, FamilyOS, Unit};

#[derive(Debug, Clone, Serialize)]
pub struct Plugin {
    #[serde(flatten)]
    plugin: Unit,
    #[serde(skip)]
    core: Core,
}

impl Plugin {
    pub fn new(name: impl Into<String>, core: &Core) -> Self {
        let mut unit = Unit::new(name, core.builder());
        unit.dependency(Dependency::Core(core.clone()));

        Self {
            plugin: unit,
            core: core.clone(),
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
        let common_dir = extracted_dir.join("src").join("common");
        let osx_dir = extracted_dir.join("src").join("osx");
        let unix_dir = extracted_dir.join("src").join("unix");
        let win_dir = extracted_dir.join("src").join("win");

        let extracted_dir_name = format!("{{sources}}/extracted/plugins/{}", &name);
        let common_sources = format!("{}/src/common/*.c", &extracted_dir_name);
        let osx_sources = format!("{}/src/osx/*.c", &extracted_dir_name);
        let unix_sources = format!("{}/src/unix/*.c", &extracted_dir_name);
        let win_sources = format!("{}/src/win/*.c", &extracted_dir_name);

        if common_dir.exists() {
            plugin.source(&common_sources);
        }

        match core.builder().target_family() {
            FamilyOS::Apple => {
                if osx_dir.exists() {
                    plugin.source(&osx_sources);
                    // If MacOS specific version does not exist we add unix for MacOS
                } else if unix_dir.exists() {
                    plugin.source(&unix_sources);
                }
            }
            FamilyOS::Unix | FamilyOS::Other => {
                if unix_dir.exists() {
                    plugin.source(&unix_sources);
                }
            }
            FamilyOS::Windows => {
                if win_dir.exists() {
                    plugin.source(&win_sources);
                }
            }
        }

        plugin.with_default_includes();

        plugin
    }

    pub fn get_includes(&self) -> &Vec<String> {
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

        let common_include_dir = extracted_dir.join("include").join("common");
        let common_src_dir = extracted_dir.join("src").join("common");
        let osx_include_dir = extracted_dir.join("include").join("osx");
        let osx_src_dir = extracted_dir.join("include").join("osx");
        let unix_include_dir = extracted_dir.join("include").join("unix");
        let unix_src_dir = extracted_dir.join("src").join("unix");
        let win_include_dir = extracted_dir.join("include").join("win");
        let win_src_dir = extracted_dir.join("src").join("win");

        let extracted_dir_name = format!("{{sources}}/extracted/plugins/{}", &self.plugin.name());
        let common_includes = format!("{}/include/common", &extracted_dir_name);
        let common_sources = format!("{}/src/common", &extracted_dir_name);
        let osx_includes = format!("{}/include/osx", &extracted_dir_name);
        let osx_sources = format!("{}/src/osx", &extracted_dir_name);
        let unix_includes = format!("{}/include/unix", &extracted_dir_name);
        let unix_sources = format!("{}/src/unix", &extracted_dir_name);
        let win_includes = format!("{}/include/win", &extracted_dir_name);
        let win_sources = format!("{}/src/win", &extracted_dir_name);

        if common_include_dir.exists() {
            self.include(&common_includes);
        }
        if common_src_dir.exists() {
            self.include(&common_sources);
        }

        match self.builder().target_family() {
            FamilyOS::Apple => {
                if osx_include_dir.exists() {
                    self.include(&osx_includes);
                } else if unix_include_dir.exists() {
                    self.include(&unix_includes);
                }
                if osx_src_dir.exists() {
                    self.include(&osx_sources);
                } else if unix_src_dir.exists() {
                    self.include(&unix_sources);
                }
            }
            FamilyOS::Unix | FamilyOS::Other => {
                if unix_include_dir.exists() {
                    self.include(&unix_includes);
                }
                if unix_src_dir.exists() {
                    self.include(&unix_sources);
                }
            }
            FamilyOS::Windows => {
                if win_include_dir.exists() {
                    self.include(&win_includes);
                }
                if win_src_dir.exists() {
                    self.include(&win_sources);
                }
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

    fn add_include(&mut self, dir: impl AsRef<str>) -> &mut Self {
        self.plugin.add_include(dir);
        self
    }

    fn add_source(&mut self, dir: impl AsRef<str>) -> &mut Self {
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
