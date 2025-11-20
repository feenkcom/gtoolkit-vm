use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fmt::format;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use anyhow::{bail, Result};
use cc::Build;
use file_matcher::FilesNamed;
use fs_extra::dir::CopyOptions;
use fs_extra::error::ErrorKind::OsString;
use new_string_template::template::Template;
use serde::{Serialize, Serializer};
use to_absolute::canonicalize;

use crate::build_support::{Core, Plugin};
use crate::{
    ArchBits, Builder, FamilyOS, Feature, HeaderDetector, TargetOS, IOS_DEPLOYMENT_TARGET,
    MACOSX_DEPLOYMENT_TARGET,
};

pub trait CompilationUnit {
    fn name(&self) -> &str;
    fn builder(&self) -> Rc<dyn Builder>;
    fn binary_name(&self) -> String {
        match self.family() {
            FamilyOS::Unix | FamilyOS::Other => format!("lib{}.so", self.name()),
            FamilyOS::Apple => format!("lib{}.dylib", self.name()),
            FamilyOS::Windows => format!("{}.dll", self.name()),
        }
    }

    fn target(&self) -> TargetOS {
        self.builder().target()
    }

    fn family(&self) -> FamilyOS {
        self.builder().target_family()
    }

    fn arch_bits(&self) -> ArchBits {
        self.builder().arch_bits()
    }

    fn output_directory(&self) -> PathBuf {
        self.builder().output_directory()
    }

    fn artefact_directory(&self) -> PathBuf {
        self.builder().artefact_directory()
    }

    fn add_include(&mut self, dir: impl AsRef<str>) -> &mut Self;
    fn add_includes<P>(&mut self, dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<str>,
    {
        for dir in dirs {
            self.add_include(dir);
        }
        self
    }

    fn include(&mut self, include: impl AsRef<str>) -> &mut Self {
        self.add_include(include);
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

    fn add_source(&mut self, dir: impl AsRef<str>) -> &mut Self;
    fn add_sources<P>(&mut self, files: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: AsRef<str>,
    {
        for file in files {
            self.add_source(file);
        }
        self
    }

    /// Add all source files matching a wildmatch template path
    fn source(&mut self, sources: impl AsRef<str>) -> &mut Self {
        self.add_source(sources);
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
        let has_header = HeaderDetector::new(header_name.as_ref()).exists();
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

#[derive(Debug, Clone, Serialize)]
pub enum Dependency {
    #[serde(serialize_with = "core_dependency")]
    Core(Core),
    #[serde(serialize_with = "plugin_dependency")]
    Plugin(Plugin),
    #[serde(serialize_with = "feature_dependency")]
    Feature(Feature),
    SystemLibrary(String),
    #[serde(serialize_with = "library_dependency")]
    Library(String, Vec<PathBuf>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Unit {
    #[serde(skip)]
    builder: Rc<dyn Builder>,
    name: String,
    includes: Vec<String>,
    sources: Vec<String>,
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
        let original_sources = find_all_sources(&self.sources, self.builder.clone()).unwrap();
        for source in &original_sources {
            println!("cargo:rerun-if-changed={}", source.display());
        }

        let mut sources = Vec::new();
        let dst = self.output_directory();
        for file in original_sources.iter() {
            let obj = dst.join(file);
            let obj = if !obj.starts_with(&dst) {
                let mut source = obj
                    .strip_prefix(self.builder.vm_sources_directory())
                    .unwrap_or_else(|_| obj.strip_prefix(std::env::current_dir().unwrap()).unwrap())
                    .to_path_buf();

                let extension = source
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_string());
                if let Some(extension) = extension.as_ref() {
                    source = match extension.as_str() {
                        "S" => source.with_extension("asm"),
                        "m" => {
                            let new_name =
                                format!("{}-mac.m", source.file_stem().unwrap().to_str().unwrap());
                            source.with_file_name(new_name)
                        }
                        _ => source,
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

        let includes = find_all_includes(&self.includes, self.builder.clone());

        let mut build = Build::new();
        build
            .cargo_metadata(false)
            .static_crt(true)
            .shared_flag(true)
            .pic(true)
            .files(sources)
            .includes(&includes)
            .warnings(false)
            .extra_warnings(false);

        let target = self.target();

        if target.is_apple() {
            if target.is_macos() {
                build.flag(&format!(
                    "-mmacosx-version-min={}",
                    MACOSX_DEPLOYMENT_TARGET
                ));
                build.flag("-stdlib=libc++");
            } else if target.is_ios() {
                build.flag(&format!("-miphoneos-version-min={}", IOS_DEPLOYMENT_TARGET));
            } else {
                eprintln!("Apple deployment target is not specified for {}", &target);
            }
        }

        for flag in &self.flags {
            build.flag_if_supported(flag);
        }

        for define in &self.defines {
            build.define(&define.0, define.1.as_ref().map(|value| value.as_str()));
        }

        let compiler = build.get_compiler();

        let is_debug = match env::var_os("DEBUG") {
            Some(s) => s != "false",
            None => false,
        };

        if is_debug {
            // force frame pointer
            if compiler.is_like_msvc() {
                // We set our own debug flags, because we want to generate .pdb
                build.debug(false);
                build.flag("/DEBUG:FULL");
                build.flag("/Zi");
                build.flag("/Oy-");
            }
        }

        // the `cc` crate create a static library by default.
        // In the case of msvc, we should override the used archiver from lib.exe to link.exe and
        // correctly provide the linking libraries from the dependencies
        if compiler.is_like_msvc() {
            let target = env::var("TARGET").expect("Could not find TARGET env.var.");
            let linker = cc::windows_registry::find_tool(&target, "link.exe")
                .expect("Could not find link.exe");
            build.archiver(linker.path());
            build.ar_flag("-DLL");

            if self.builder().is_debug() {
                build.ar_flag("-DEBUG");
            }

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
                    Dependency::Feature(_) => { /* nothing to do here */ }
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

            let is_gcc = compiler.is_like_gnu() && !self.target().is_apple();
            let is_clang = compiler.is_like_clang() || self.target().is_apple();

            if self.target().is_apple() {
                // allows the header to be changed later on during packaging
                command.arg("-headerpad_max_install_names");
            }

            // there is a difference in how clang and gnu
            if is_gcc {
                command.arg("-Wl,--whole-archive");
            }
            if is_clang {
                // Android's clang does not support -all_load
                if self.target().is_android() {
                    command.arg("-Wl,--whole-archive");
                    command.arg("-Wl,--allow-multiple-definition");
                } else {
                    command.arg("-Wl,-all_load");
                }
            }
            command.arg(format!("lib{}.a", self.name()));

            if is_gcc {
                command.arg("-Wl,--no-whole-archive");
            }

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
                    Dependency::Feature(_) => { /* nothing to do here */ }
                }
            }

            command.arg("-o").arg(self.binary_name());

            if !command.status().unwrap().success() {
                panic!("Failed to create {}", self.binary_name());
            }
        }

        if !self.artefact_directory().exists() {
            std::fs::create_dir_all(self.artefact_directory()).unwrap();
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

        if is_debug && self.target().is_macos() {
            let dylib = self.output_directory().join(self.binary_name());
            let dylib_path_str = dylib.display().to_string();
            let status = Command::new("dsymutil")
                .args([
                    dylib_path_str.as_str(),
                    "-o",
                    &format!("{}.dSYM", dylib.display()),
                ])
                .status()
                .expect("failed to run dsymutil");
            if !status.success() {
                println!("cargo:warning=dsymutil failed to generate debug symbols");
            }

            if self.target().is_macos() {
                let copy_options = CopyOptions::default().overwrite(true);
                fs_extra::dir::copy(
                    self.output_directory()
                        .join(format!("{}.dSYM", self.binary_name())),
                    self.artefact_directory(),
                    &copy_options,
                )
                .unwrap();
            }
        }

        if is_debug && compiler.is_like_msvc() {
            std::fs::copy(
                self.output_directory().join(format!("{}.pdb", self.name())),
                self.artefact_directory()
                    .join(format!("{}.pdb", self.name())),
            )
            .unwrap();
        }

        build
    }

    pub fn get_sources(&self) -> &Vec<String> {
        &self.sources
    }

    pub fn get_includes(&self) -> &Vec<String> {
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

    fn add_include(&mut self, dir: impl AsRef<str>) -> &mut Self {
        self.includes.push(dir.as_ref().to_string());
        self
    }

    fn add_source(&mut self, dir: impl AsRef<str>) -> &mut Self {
        self.sources.push(dir.as_ref().to_string());
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

fn find_all_sources(
    sources_wildmatch: &Vec<String>,
    builder: Rc<dyn Builder>,
) -> Result<Vec<PathBuf>> {
    let mut sources = vec![];
    for wildmatch in sources_wildmatch {
        sources.extend(find_sources(wildmatch, builder.clone())?);
    }
    Ok(sources)
}

fn find_sources(sources_wildmatch: &str, builder: Rc<dyn Builder>) -> Result<Vec<PathBuf>> {
    let path = template_string_to_path(sources_wildmatch, builder);
    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let directory = path.parent().unwrap().to_path_buf();

    let files = FilesNamed::wildmatch(&file_name)
        .within(&directory)
        .find()
        .unwrap();
    if files.is_empty() {
        bail!(
            "Could not find files matching {} in {}",
            &file_name,
            directory.display()
        );
    }
    Ok(files)
}

fn find_all_includes(includes: &Vec<String>, builder: Rc<dyn Builder>) -> Vec<PathBuf> {
    includes
        .iter()
        .map(|each| template_string_to_path(each.as_str(), builder.clone()))
        .filter(|each| each.exists())
        .map(|each| canonicalize(each).unwrap())
        .collect()
}

fn template_string_to_path(template_path: &str, builder: Rc<dyn Builder>) -> PathBuf {
    let template = Template::new(template_path);
    let mut data = HashMap::<String, String>::new();
    data.insert(
        "generated".to_string(),
        builder.generated_directory().display().to_string(),
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
    data.insert(
        "crate".to_string(),
        std::env::current_dir().unwrap().display().to_string(),
    );
    let rendered = template.render_string(&data).unwrap();
    PathBuf::from(rendered)
}

fn core_dependency<S>(core: &Core, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(core.name())
}

fn plugin_dependency<S>(plugin: &Plugin, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(plugin.name())
}

fn feature_dependency<S>(feature: &Feature, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(feature.name())
}

fn library_dependency<S>(
    name: &String,
    _links: &Vec<PathBuf>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(name.as_str())
}
