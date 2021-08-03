use crate::libraries::library::{TarArchive, TarUrlLocation};
use crate::options::BundleOptions;
use crate::{Library, LibraryLocation, NativeLibrary};
use std::error::Error;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use user_error::UserFacingError;

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

    fn patch_windows_makefile(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        if self
            .source_directory(options)
            .join("Makefile.win32.common.fixed")
            .exists()
        {
            return Ok(());
        }

        let makefile = self.source_directory(options).join("Makefile.win32.common");
        std::fs::copy(
            &makefile,
            self.source_directory(options)
                .join("Makefile.win32.common.bak"),
        )?;

        let mut contents = read_to_string(&makefile)?;
        contents = contents.replace("-MD", "-MT");

        let include_flags_to_replace =
            "BASE_CFLAGS = -nologo -I. -I$(top_srcdir) -I$(top_srcdir)/pixman";
        let new_include_flags = self
            .msvc_include_directories()
            .into_iter()
            .map(|path| format!("BASE_CFLAGS += -I\"{}\"", path.display()))
            .collect::<Vec<String>>()
            .join("\n");

        contents = contents.replace(
            include_flags_to_replace,
            &format!("{}\n{}", include_flags_to_replace, new_include_flags),
        );

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&makefile)?;
        file.write(contents.as_bytes())?;

        std::fs::copy(
            &makefile,
            self.source_directory(options)
                .join("Makefile.win32.common.fixed"),
        )?;

        Ok(())
    }

    fn compile_unix(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        self.patch_makefile(options)?;

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

        let configure = command.status()?;

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }

        let make = Command::new("make")
            .current_dir(&makefile_dir)
            .arg("install")
            .status()?;

        if !make.success() {
            panic!("Could not compile {}", self.name());
        }
        Ok(())
    }

    fn compile_windows(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        self.patch_makefile(options)
            .expect("Failed to patch a Makefile");

        self.patch_windows_makefile(options)
            .expect("Failed to patch a Windows specific Makefile");

        let makefile = self.source_directory(options).join("Makefile.win32");

        let mut command = Command::new("make");
        command
            .current_dir(self.source_directory(options))
            .arg("pixman")
            .arg("-f")
            .arg(&makefile)
            .arg("CFG=release")
            .arg("MMX=off");

        println!("{:?}", &command);

        let configure = command.status().unwrap();

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }
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
        if options.target().is_unix() {
            self.compile_unix(options)
                .expect("Failed to compile pixman")
        }
        if options.target().is_windows() {
            self.compile_windows(options)
                .expect("Failed to compile pixman")
        }
    }

    fn compiled_library_directories(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        unimplemented!()
    }

    fn compiled_library_binary(&self, options: &BundleOptions) -> Result<PathBuf, Box<dyn Error>> {
        if options.target().is_windows() {
            return Ok(self
                .source_directory(options)
                .join("pixman")
                .join(options.profile())
                .join("pixman-1.lib"));
        }
        Err(UserFacingError::new("Could not find compiled library").into())
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool {
        false
    }

    fn ensure_requirements(&self, options: &BundleOptions) {
        which::which("make").expect("Could not find `make`");
        which::which("autoreconf").expect("Could not find `make`");

        if options.target().is_windows() {
            which::which("coreutils").expect("Could not find `coreutils`");

            for path in self.msvc_lib_directories() {
                if !path.exists() {
                    panic!("Lib folder does not exist: {}", &path.display())
                }
            }
            for path in self.msvc_include_directories() {
                if !path.exists() {
                    panic!("Include folder does not exist: {}", &path.display())
                }
            }
        }
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl NativeLibrary for PixmanLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        if options.target().is_unix() {
            return options.target_dir().join(self.name());
        }
        if options.target().is_windows() {
            return self.source_directory(options);
        }
        panic!("Unknown platform!")
    }

    fn native_library_dependency_prefixes(&self, _options: &BundleOptions) -> Vec<PathBuf> {
        vec![]
    }

    fn native_library_include_headers(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let library_prefix = self.native_library_prefix(options);
        if options.target().is_unix() {
            return vec![library_prefix.join("include").join("pixman-1")];
        }
        if options.target().is_windows() {
            return vec![library_prefix];
        }
        vec![]
    }

    fn native_library_linker_libraries(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let library_prefix = self.native_library_prefix(options);
        if options.target().is_unix() {
            return vec![library_prefix.join("lib")];
        }
        if options.target().is_windows() {
            return vec![library_prefix.join("pixman").join(options.profile())];
        }
        vec![]
    }

    fn clone_native_library(&self) -> Box<dyn NativeLibrary> {
        Box::new(Clone::clone(self))
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
