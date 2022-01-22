use crate::build_support::{Core, Plugin};
use crate::{Builder, BuilderTarget};
use cc::Build;
use file_matcher::FilesNamed;
use new_string_template::template::Template;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use to_absolute::canonicalize;

pub trait CompilationUnit {
    fn name(&self) -> &str;
    fn builder(&self) -> Rc<dyn Builder>;
    fn binary_name(&self) -> String {
        match self.target() {
            BuilderTarget::Linux => format!("lib{}.so", self.name()),
            BuilderTarget::MacOS => format!("lib{}.dylib", self.name()),
            BuilderTarget::Windows => format!("{}.dll", self.name()),
        }
    }
    fn target(&self) -> BuilderTarget {
        self.builder().target()
    }

    fn output_directory(&self) -> PathBuf {
        self.builder().output_directory()
    }

    fn artefact_directory(&self) -> PathBuf {
        self.builder().artefact_directory()
    }

    fn add_include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;
    fn add_includes<P>(&mut self, dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for dir in dirs {
            self.add_include(dir);
        }
        self
    }

    fn include(&mut self, include: impl AsRef<str>) -> &mut Self {
        let path = template_string_to_path(include.as_ref(), self.builder());
        self.add_include(path);
        self
    }

    fn includes<P>(&mut self, includes: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<str>,
    {
        for include in includes {
            self.include(include);
        }
        self
    }

    fn add_source<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;
    fn add_sources<P>(&mut self, files: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for file in files {
            self.add_source(file);
        }
        self
    }

    /// Add all source files matching a wildmatch template path
    fn source(&mut self, sources: impl AsRef<str>) -> &mut Self {
        let path = template_string_to_path(sources.as_ref(), self.builder());
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let directory = path.parent().unwrap().to_path_buf();

        let files = FilesNamed::wildmatch(&file_name)
            .within(&directory)
            .find()
            .unwrap();
        if files.is_empty() {
            panic!(
                "Could not find files matching {} in {}",
                &file_name,
                directory.display()
            );
        }
        self.add_sources(files);
        self
    }

    fn sources<S>(&mut self, sources: S) -> &mut Self
    where
        S: IntoIterator,
        S::Item: AsRef<str>,
    {
        for source in sources {
            self.source(source);
        }
        self
    }

    fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) -> &mut Self;

    /// Checks if a provided header exists, and if it does define a symbol
    fn define_for_header(
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

    fn dependency(&mut self, dependency: Dependency) -> &mut Self;
    fn dependencies<D>(&mut self, dependencies: D) -> &mut Self
    where
        D: IntoIterator<Item = Dependency>,
    {
        for dependency in dependencies {
            self.dependency(dependency);
        }
        self
    }
}

#[derive(Debug, Clone)]
pub enum Dependency {
    Core(Core),
    Plugin(Plugin),
    SystemLibrary(String),
    Library(String, Vec<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct Unit {
    builder: Rc<dyn Builder>,
    name: String,
    includes: Vec<PathBuf>,
    sources: Vec<PathBuf>,
    defines: Vec<(String, Option<String>)>,
    flags: Vec<String>,
    dependencies: Vec<Dependency>,
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
            dependencies: vec![],
        }
    }

    pub fn compile(&self) -> Build {
        let original_sources = &self.sources;
        let mut sources = Vec::new();
        let dst = self.output_directory();
        for file in original_sources.iter() {
            let obj = dst.join(file);
            let obj = if !obj.starts_with(&dst) {
                let mut source = obj
                    .strip_prefix(self.builder.vm_sources_directory())
                    .unwrap()
                    .to_path_buf();

                if let Some(extension) = source.extension() {
                    if extension == OsStr::new("S") {
                        source = source.with_extension("asm");
                    }
                }

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
            .cargo_metadata(false)
            .static_crt(true)
            .shared_flag(true)
            .pic(true)
            .files(sources)
            .includes(&self.includes)
            .warnings(false)
            .extra_warnings(false);

        for flag in &self.flags {
            build.flag_if_supported(flag);
        }

        for define in &self.defines {
            build.define(&define.0, define.1.as_ref().map(|value| value.as_str()));
        }

        let compiler = build.get_compiler();
        // the `cc` crate create a static library by default. In case of msvc we should override the used archiver from lib.exe to link.exe and
        // correctly provide the linking libraries from the dependencies
        if compiler.is_like_msvc() {
            build.archiver("link.exe");
            build.ar_flag("-DLL");
            let libs = compiler
                .env()
                .iter()
                .find(|var| var.0 == OsStr::new("LIB"))
                .expect("MSVC toolchain is not detected");
            for path in std::env::split_paths(&libs.1) {
                if let Ok(path) = to_absolute::canonicalize(path) {
                    build.ar_flag(&format!("-LIBPATH:{}\\", path.display()));
                }
            }
            for dependency in &self.dependencies {
                match dependency {
                    Dependency::Core(core) => {
                        build.ar_flag(&format!("{}.lib", core.name()));
                        build.ar_flag(&format!(
                            "-LIBPATH:{}\\",
                            to_absolute::canonicalize(core.artefact_directory())
                                .unwrap()
                                .display()
                        ));
                    }
                    Dependency::Plugin(plugin) => {
                        build.ar_flag(&format!("{}.lib", plugin.name()));
                        build.ar_flag(&format!(
                            "-LIBPATH:{}\\",
                            to_absolute::canonicalize(plugin.artefact_directory())
                                .unwrap()
                                .display()
                        ));
                    }
                    Dependency::SystemLibrary(framework) => {
                        build.ar_flag(&format!("{}.lib", framework));
                    }
                    Dependency::Library(library, link_path) => {
                        for path in link_path {
                            build.ar_flag(&format!(
                                "-LIBPATH:{}\\",
                                to_absolute::canonicalize(path).unwrap().display()
                            ));
                        }
                        build.ar_flag(&format!("{}.lib", library));
                    }
                }
            }
        }

        if compiler.is_like_msvc() {
            build
                .try_compile_binary(self.name(), self.binary_name().as_str())
                .unwrap();
        } else {
            build.compile(self.name());
        }

        if !compiler.is_like_msvc() {
            let mut command = compiler.to_command();
            command.current_dir(self.output_directory());
            command
                .arg("-Wl,-all_load")
                .arg(format!("lib{}.a", self.name()));

            for dependency in &self.dependencies {
                match dependency {
                    Dependency::Core(unit) => {
                        command
                            .arg("-L")
                            .arg(self.artefact_directory())
                            .arg("-l")
                            .arg(unit.name());
                    }
                    Dependency::Plugin(unit) => {
                        command
                            .arg("-L")
                            .arg(self.artefact_directory())
                            .arg("-l")
                            .arg(unit.name());
                    }
                    Dependency::SystemLibrary(framework) => {
                        command.arg("-framework").arg(framework);
                    }
                    Dependency::Library(library, link_path) => {
                        for path in link_path {
                            command.arg("-L").arg(path);
                        }
                        command.arg("-l").arg(library);
                    }
                }
            }

            command.arg("-o").arg(self.binary_name());

            if !command.status().unwrap().success() {
                panic!("Failed to create {}", self.binary_name());
            }
        }

        std::fs::copy(
            self.output_directory().join(self.binary_name()),
            self.artefact_directory().join(self.binary_name()),
        )
        .unwrap();

        if compiler.is_like_msvc() {
            std::fs::copy(
                self.output_directory().join(format!("{}.lib", self.name())),
                self.artefact_directory()
                    .join(format!("{}.lib", self.name())),
            )
            .unwrap();
            std::fs::copy(
                self.output_directory().join(format!("{}.exp", self.name())),
                self.artefact_directory()
                    .join(format!("{}.exp", self.name())),
            )
            .unwrap();
        }

        build
    }

    pub fn get_sources(&self) -> &Vec<PathBuf> {
        &self.sources
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

    pub fn get_dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }

    pub fn merge(&self, unit: &Unit) -> Unit {
        let mut combined = self.clone();
        combined.add_sources(unit.get_sources());
        combined.add_includes(unit.get_includes());
        for define in unit.get_defines() {
            combined.define(
                define.0.as_str(),
                define.1.as_ref().map(|value| value.as_str()),
            );
        }
        combined.flags(unit.get_flags());
        combined.dependencies(unit.get_dependencies().clone());
        combined
    }
}

impl CompilationUnit for Unit {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn builder(&self) -> Rc<dyn Builder> {
        self.builder.clone()
    }

    fn add_include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        let path = dir.as_ref().to_path_buf();
        if path.exists() {
            let path = canonicalize(path).unwrap();
            self.includes.push(path);
        } else {
            eprintln!("Include path does not exist: {}", &path.display());
        }
        self
    }

    fn add_source<P: AsRef<Path>>(&mut self, file: P) -> &mut Self {
        let path = file.as_ref().to_path_buf();
        if path.exists() {
            self.sources.push(path);
        }
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

    fn dependency(&mut self, dependency: Dependency) -> &mut Self {
        self.dependencies.push(dependency);
        self
    }
}

fn template_string_to_path(template_path: &str, builder: Rc<dyn Builder>) -> PathBuf {
    let template = Template::new(template_path);
    let mut data = HashMap::<String, String>::new();
    data.insert(
        "generated".to_string(),
        builder
            .output_directory()
            .join("generated")
            .join("64")
            .display()
            .to_string(),
    );
    data.insert(
        "output".to_string(),
        builder.output_directory().display().to_string(),
    );
    data.insert(
        "sources".to_string(),
        builder.vm_sources_directory().display().to_string(),
    );
    data.insert("profile".to_string(), builder.profile());
    let rendered = template.render_string(&data).unwrap();
    PathBuf::from(rendered)
}
