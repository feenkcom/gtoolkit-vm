use std::{env, fs};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HeaderDetector {
    header: String,
}

impl HeaderDetector {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
        }
    }

    pub fn exists(&self) -> bool {
        let env_var = env::var("OUT_DIR").unwrap();
        let out_dir = Path::new(env_var.as_str());
        let header_dir = out_dir.join("header_detection");
        if header_dir.exists() {
            fs::remove_dir_all(&header_dir).expect(&format!("Remove {}", header_dir.display()));
        }
        fs::create_dir_all(&header_dir).expect(&format!("Create {}", header_dir.display()));

        let source = format!("#include <{}>", self.header.as_str());
        let source_file = header_dir.join("main.c");
        fs::write(&source_file, &source).expect(&format!(
            "Write {} to {}",
            source.as_str(),
            source_file.display()
        ));

        let mut build = cc::Build::new();
        build
            .out_dir(&header_dir)
            .file(&source_file)
            .cargo_metadata(false)
            .try_compile("main")
            .map(|_| true)
            .unwrap_or_else(|error| {
                println!("Trying to detect header {} resulted in {}", &self.header, error);
                false
            })
    }
}
