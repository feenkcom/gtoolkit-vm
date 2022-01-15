use crate::{Builder, BuilderTarget};
use cc::Build;
use file_matcher::{ManyEntries, OneEntry};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub trait CompilationUnit {
    fn name(&self) -> &str;
    fn builder(&self) -> Rc<dyn Builder>;
    fn binary_name(&self) -> String {
        match self.builder().target() {
            BuilderTarget::Linux => format!("lib{}.so", self.name()),
            BuilderTarget::MacOS => format!("lib{}.dylib", self.name()),
            BuilderTarget::Windows => format!("{}.dll", self.name()),
        }
    }

    fn output_directory(&self) -> PathBuf {
        self.builder().output_directory()
    }

    fn artefact_directory(&self) -> PathBuf {
        self.builder().artefact_directory()
    }

    fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;
    fn includes<P>(&mut self, dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for dir in dirs {
            self.include(dir);
        }
        self
    }

    fn file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;
    fn files<P>(&mut self, dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for dir in dirs {
            self.file(dir);
        }
        self
    }
    fn files_named(&mut self, files: ManyEntries) -> &mut Self {
        self.files(files.find().unwrap())
    }
    fn file_named(&mut self, file: OneEntry) -> &mut Self {
        self.file(file.find().unwrap())
    }
    fn remove_file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self;
    fn flag(&mut self, flag: &str) -> &mut Self;
    fn flags<P>(&mut self, flags: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<str>,
    {
        for flag in flags {
            self.flag(flag.as_ref());
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    builder: Rc<dyn Builder>,
    name: String,
    includes: Vec<PathBuf>,
    sources: Vec<PathBuf>,
    defines: Vec<(String, Option<String>)>,
    flags: Vec<String>,
}

impl Unit {
    pub fn new(name: impl Into<String>, builder: Rc<dyn Builder>) -> Self {
        Self {
            builder,
            name: name.into(),
            includes: vec![],
            sources: vec![],
            defines: Vec::new(),
            flags: vec![],
        }
    }

    pub fn compile(&self) -> Build {
        let original_sources = &self.sources;
        let mut sources = Vec::new();
        let dst = self.output_directory();
        for file in original_sources.iter() {
            let obj = dst.join(file);
            let obj = if !obj.starts_with(&dst) {
                let source = obj
                    .strip_prefix(self.builder.vm_sources_directory())
                    .unwrap();
                let dst_source = dst.join(source);
                std::fs::create_dir_all(dst_source.parent().unwrap()).unwrap();
                std::fs::copy(file, &dst_source).unwrap();
                dst_source
            } else {
                obj
            };
            sources.push(obj);
        }

        let mut build = cc::Build::new();
        build
            .static_crt(true)
            .shared_flag(true)
            .pic(true)
            .files(sources)
            .includes(&self.includes)
            .warnings(false)
            .extra_warnings(false);

        for flag in &self.flags {
            build.flag(flag);
        }

        for define in &self.defines {
            build.define(&define.0, define.1.as_ref().map(|value| value.as_str()));
        }

        build.compile(&self.name);
        build
    }

    pub fn get_includes(&self) -> &Vec<PathBuf> {
        &self.includes
    }

    pub fn get_defines(&self) -> &Vec<(String, Option<String>)> {
        &self.defines
    }

    pub fn get_flags(&self) -> &Vec<String> {
        &self.flags
    }
}

impl CompilationUnit for Unit {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn builder(&self) -> Rc<dyn Builder> {
        self.builder.clone()
    }

    fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.includes.push(dir.as_ref().to_path_buf());
        self
    }

    fn file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.sources.push(dir.as_ref().to_path_buf());
        self
    }

    fn remove_file<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        let path_to_remove = dir.as_ref().to_path_buf();
        self.sources.retain(|each| each != &path_to_remove);
        self
    }

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self {
        self.defines
            .push((var.to_string(), val.into().map(|s| s.to_string())));
        self
    }

    fn flag(&mut self, flag: &str) -> &mut Self {
        self.flags.push(flag.to_string());
        self
    }
}
