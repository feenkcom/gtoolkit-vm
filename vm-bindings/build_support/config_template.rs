use new_string_template::template::Template;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub struct ConfigTemplate {
    config: PathBuf,
    output: PathBuf,
    data: HashMap<String, String>,
}

impl ConfigTemplate {
    pub fn new(template: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            config: template.into(),
            output: output.into(),
            data: HashMap::new(),
        }
    }

    pub fn var(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.data.insert(key.into(), value.into());
        self
    }

    pub fn render(&self) {
        let mut output = File::create(&self.output).unwrap();
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
