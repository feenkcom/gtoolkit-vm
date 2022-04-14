use crate::Builder;
use new_string_template::template::Template;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::string::ToString;
use strum::{Display, EnumVariantNames, VariantNames};

#[allow(non_camel_case_types)]
#[derive(Display, EnumVariantNames, Debug, Clone, Serialize)]
pub enum Config {
    VM_NAME(String),
    DEFAULT_IMAGE_NAME(String),
    OS_TYPE(String),
    VM_TARGET(String),
    VM_TARGET_OS(String),
    VM_TARGET_CPU(String),
    SIZEOF_INT(usize),
    SIZEOF_LONG(usize),
    SIZEOF_LONG_LONG(usize),
    SIZEOF_VOID_P(usize),
    SQUEAK_INT64_TYPEDEF(String),
    VERSION_MAJOR(usize),
    VERSION_MINOR(usize),
    VERSION_PATCH(usize),
    BUILT_FROM(String),
    ALWAYS_INTERACTIVE(bool),
}

impl Config {
    pub fn value_to_string(&self) -> String {
        match self {
            Config::VM_NAME(value) => value.to_string(),
            Config::DEFAULT_IMAGE_NAME(value) => value.to_string(),
            Config::OS_TYPE(value) => value.to_string(),
            Config::VM_TARGET(value) => value.to_string(),
            Config::VM_TARGET_OS(value) => value.to_string(),
            Config::VM_TARGET_CPU(value) => value.to_string(),
            Config::SIZEOF_INT(value) => value.to_string(),
            Config::SIZEOF_LONG(value) => value.to_string(),
            Config::SIZEOF_LONG_LONG(value) => value.to_string(),
            Config::SIZEOF_VOID_P(value) => value.to_string(),
            Config::SQUEAK_INT64_TYPEDEF(value) => value.to_string(),
            Config::VERSION_MAJOR(value) => value.to_string(),
            Config::VERSION_MINOR(value) => value.to_string(),
            Config::VERSION_PATCH(value) => value.to_string(),
            Config::BUILT_FROM(value) => value.to_string(),
            Config::ALWAYS_INTERACTIVE(value) => (if *value { 1 } else { 0 }).to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigTemplate {
    #[serde(skip)]
    config: PathBuf,
    #[serde(skip)]
    output: PathBuf,
    data: HashMap<String, String>,
}

impl ConfigTemplate {
    pub fn new(builder: Rc<dyn Builder>) -> Self {
        let config = builder
            .vm_sources_directory()
            .join("include")
            .join("pharovm")
            .join("config.h.in");
        let output = builder
            .output_directory()
            .join("generated")
            .join("64")
            .join("vm")
            .join("include")
            .join("config.h");
        Self {
            config,
            output,
            data: HashMap::new(),
        }
    }

    pub fn var(&mut self, config: Config) -> &mut Self {
        let key = config.to_string();
        let value = config.value_to_string();
        self.data.insert(key, value);
        self
    }

    pub fn render(&self) {
        let keys: Vec<String> = Config::VARIANTS
            .iter()
            .map(|each| each.to_string())
            .filter(|each| !self.data.contains_key(each))
            .collect::<Vec<String>>();
        if !keys.is_empty() {
            panic!("Some config values are not defined: {:?}", keys);
        }

        let mut output = File::create(&self.output).unwrap_or_else(|error| {
            panic!("Failed to create file named {}: {}", &self.output.display(), error);
        });
        let custom_regex = Regex::new(r"(?mi)@+([^@]+)@").unwrap();

        if let Ok(lines) = Self::read_lines(&self.config) {
            // Consumes the iterator, returns an (Optional) String
            for line in lines {
                if let Ok(line) = line {
                    if !line.contains("cmakedefine") {
                        let template = Template::new(line).with_regex(&custom_regex);
                        let rendered = template.render_string(&self.data).unwrap();
                        writeln!(output, "{}", rendered).unwrap();
                    }
                }
            }
        }
    }

    // The output is wrapped in a Result to allow matching on errors
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}
