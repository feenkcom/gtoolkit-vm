use crate::libraries::library::{TarArchive, TarUrlLocation};
use crate::options::{BundleOptions, Target};
use crate::{
    freetype_static, pixman, png_static, Library, LibraryLocation, NativeLibrary,
    NativeLibraryDependencies,
};
use std::env::VarError;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct CairoLibrary {
    location: LibraryLocation,
    dependencies: NativeLibraryDependencies,
}

impl CairoLibrary {
    pub fn new() -> Self {
        Self {
            location: LibraryLocation::Tar(
                TarUrlLocation::new("https://cairographics.org/snapshots/cairo-1.17.4.tar.xz")
                    .archive(TarArchive::Xz)
                    .sources(Path::new("cairo-1.17.4")),
            ),
            dependencies: NativeLibraryDependencies::new()
                .add(pixman().into())
                .add(freetype_static().into())
                .add(png_static().into()),
        }
    }
}

impl Library for CairoLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "cairo"
    }

    fn ensure_sources(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        self.location()
            .ensure_sources(&self.source_directory(options), options)?;
        self.dependencies.ensure_sources(options)?;
        Ok(())
    }

    fn force_compile(&self, options: &BundleOptions) {
        self.dependencies.compile(options);

        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir).expect(&format!("Could not create {:?}", &out_dir));
        }
        let makefile_dir = out_dir.clone();

        let mut pkg_config_paths = self.dependencies.pkg_config_directories(options);
        pkg_config_paths.push(PathBuf::from("../pixman"));
        if let Ok(ref path) = std::env::var("PKG_CONFIG_PATH") {
            std::env::split_paths(path).for_each(|path| pkg_config_paths.push(path));
        }
        std::env::set_var(
            "PKG_CONFIG_PATH",
            std::env::join_paths(&pkg_config_paths).unwrap(),
        );

        println!("PKG_CONFIG_PATH={:?}", std::env::var("PKG_CONFIG_PATH"));
        
        let mut cpp_flags = std::env::var("CPPFLAGS").unwrap_or("".to_owned());
        cpp_flags = format!(
            "{} {}",
            cpp_flags,
            self.dependencies.include_headers_flags(options)
        );
        std::env::set_var("CPPFLAGS", &cpp_flags);
        std::env::set_var("LIBS", "-lbz2");

        let mut linker_flags = std::env::var("LDFLAGS").unwrap_or("".to_owned());
        linker_flags = format!(
            "{} {}",
            linker_flags,
            self.dependencies.linker_libraries_flags(options)
        );
        //std::env::set_var("LDFLAGS", &linker_flags);

        println!("PKG_CONFIG_PATH={:?}", std::env::var("PKG_CONFIG_PATH"));
        println!("CPPFLAGS={:?}", std::env::var("CPPFLAGS"));
        println!("LDFLAGS={:?}", std::env::var("LDFLAGS"));

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
            .arg(format!(
                "--libdir={}",
                self.native_library_prefix(options).join("lib").display()
            ));

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

    fn compiled_library_directories(&self, options: &BundleOptions) -> Vec<PathBuf> {
        let lib = self.native_library_prefix(options).join("lib");
        vec![lib]
    }

    fn ensure_requirements(&self) {
        which::which("make").expect("Could not find `make`");
    }
}

impl NativeLibrary for CairoLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        options.target_dir().join(self.name())
    }

    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf> {
        self.dependencies.dependency_prefixes(options)
    }
}

impl From<CairoLibrary> for Box<dyn Library> {
    fn from(library: CairoLibrary) -> Self {
        Box::new(library)
    }
}
