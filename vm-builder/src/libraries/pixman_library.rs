use crate::libraries::library::{TarArchive, TarUrlLocation};
use crate::options::BundleOptions;
use crate::{Library, LibraryLocation, NativeLibrary};
use std::error::Error;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PixmanLibrary {
    location: LibraryLocation,
}

impl PixmanLibrary {
    pub fn new() -> Self {
        Self {
            location: LibraryLocation::Tar(
                TarUrlLocation::new("https://www.cairographics.org/releases/pixman-0.40.0.tar.gz")
                    .archive(TarArchive::Gz)
                    .sources(Path::new("pixman-0.40.0")),
            ),
        }
    }

    fn patch_makefile(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        let makefile = self.source_directory(options).join("Makefile.in");

        let contents = read_to_string(&makefile)?;
        let new = contents.replace("SUBDIRS = pixman demos test", "SUBDIRS = pixman");
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&makefile)?;
        file.write(new.as_bytes())?;
        Ok(())
    }
}

impl Library for PixmanLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "pixman"
    }

    fn force_compile(&self, options: &BundleOptions) {
        self.patch_makefile(options)
            .expect("Failed to patch a Makefile");

        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir).expect(&format!("Could not create {:?}", &out_dir));
        }
        let makefile_dir = out_dir.clone();

        let mut command = Command::new(self.source_directory(options).join("configure"));
        command
            .current_dir(&out_dir)
            .arg(format!(
                "--prefix={}",
                self.native_library_prefix(options).display()
            ))
            .arg(format!(
                "--exec-prefix={}",
                self.native_library_prefix(options).display()
            ))
            .arg(format!("--enable-shared={}", false))
            .arg("--disable-gtk");

        println!("{:?}", &command);

        let configure = command.status().unwrap();

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }

        let make = Command::new("make")
            .current_dir(&makefile_dir)
            .arg("install")
            .status()
            .unwrap();

        if !make.success() {
            panic!("Could not compile {}", self.name());
        }
    }

    fn compiled_library_directories(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        unimplemented!()
    }

    fn ensure_requirements(&self) {
        which::which("make").expect("Could not find `make`");
        which::which("autoreconf").expect("Could not find `make`");
    }
}

impl NativeLibrary for PixmanLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        options.target_dir().join(self.name())
    }

    fn native_library_dependency_prefixes(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }

    fn native_library_include_headers(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let include_dir = self
            .native_library_prefix(options)
            .join("include")
            .join("pixman-1");
        vec![include_dir]
    }

    fn native_library_linker_libraries(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let libs_dir = self.native_library_prefix(options).join("lib");
        vec![libs_dir]
    }
}

impl From<PixmanLibrary> for Box<dyn Library> {
    fn from(library: PixmanLibrary) -> Self {
        Box::new(library)
    }
}

impl From<PixmanLibrary> for Box<dyn NativeLibrary> {
    fn from(library: PixmanLibrary) -> Self {
        Box::new(library)
    }
}
