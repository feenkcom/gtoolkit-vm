use crate::libraries::library::{TarArchive, TarUrlLocation};
use crate::options::BundleOptions;
use crate::{
    freetype_static, pixman, png_static, zlib_static, CMakeLibrary, Library, LibraryLocation,
    NativeLibrary, NativeLibraryDependencies, PixmanLibrary,
};
use std::error::Error;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CairoLibrary {
    location: LibraryLocation,
    dependencies: NativeLibraryDependencies,
    pixman: PixmanLibrary,
    zlib: CMakeLibrary,
    png: CMakeLibrary,
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
            pixman: pixman(),
            zlib: zlib_static(),
            png: png_static(),
        }
    }

    fn compile_unix(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
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

        Ok(())
    }

    fn compile_windows(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        self.patch_windows_common_makefile(options)?;
        self.patch_windows_features_makefile(options)?;
        self.patch_windows_makefile(options)?;

        let makefile = self.source_directory(options).join("Makefile.win32");

        let mut command = Command::new("make");
        command
            .current_dir(self.source_directory(options))
            .arg("cairo")
            .arg("-f")
            .arg(&makefile)
            .arg("CFG=release")
            .arg(format!(
                "PIXMAN_PATH={}",
                self.pixman.native_library_prefix(options).display()
            ))
            .arg(format!(
                "ZLIB_PATH={}",
                self.zlib.native_library_prefix(options).display()
            ))
            .arg(format!(
                "LIBPNG_PATH={}",
                self.png.native_library_prefix(options).display()
            ));

        println!("{:?}", &command);

        let configure = command.status().unwrap();

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }
        Ok(())
    }

    fn patch_windows_common_makefile(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        let makefile_directory = self.source_directory(options).join("build");

        if makefile_directory
            .join("Makefile.win32.common.fixed")
            .exists()
        {
            return Ok(());
        }

        let makefile = makefile_directory.join("Makefile.win32.common");

        std::fs::copy(
            &makefile,
            makefile_directory.join("Makefile.win32.common.bak"),
        )?;

        let mut contents = read_to_string(&makefile)?;
        contents = contents.replace("-MD", "-MT");
        contents = contents.replace(
            "CAIRO_LIBS += $(ZLIB_PATH)/zdll.lib",
            "CAIRO_LIBS += $(ZLIB_PATH)/lib/zlibstatic.lib",
        );
        contents = contents.replace(
            "ZLIB_CFLAGS += -I$(ZLIB_PATH)",
            "ZLIB_CFLAGS += -I$(ZLIB_PATH)/include",
        );
        contents = contents.replace(
            "CAIRO_LIBS +=  $(LIBPNG_PATH)/libpng.lib",
            "CAIRO_LIBS +=  $(LIBPNG_PATH)/lib/libpng16_static.lib",
        );
        contents = contents.replace(
            "LIBPNG_CFLAGS += -I$(LIBPNG_PATH)/",
            "LIBPNG_CFLAGS += -I$(LIBPNG_PATH)/include",
        );

        contents = contents.replace("@mkdir", "@coreutils mkdir");
        contents = contents.replace("`dirname $<`", "\"$(shell coreutils dirname $<)\"");

        let include_flags_to_replace = "DEFAULT_CFLAGS += -I. -I$(top_srcdir) -I$(top_srcdir)/src";
        let new_include_flags = self
            .msvc_include_directories()
            .into_iter()
            .map(|path| format!("DEFAULT_CFLAGS += -I\"{}\"", path.display()))
            .collect::<Vec<String>>()
            .join("\n");

        contents = contents.replace(
            include_flags_to_replace,
            &format!("{}\n{}", include_flags_to_replace, new_include_flags),
        );

        let ld_flags_to_replace = "DEFAULT_LDFLAGS = -nologo $(CFG_LDFLAGS)";
        let new_ld_flags = self
            .msvc_lib_directories()
            .into_iter()
            .map(|path| format!("DEFAULT_LDFLAGS += -LIBPATH:\"{}\"", path.display()))
            .collect::<Vec<String>>()
            .join("\n");

        contents = contents.replace(
            ld_flags_to_replace,
            &format!("{}\n{}", ld_flags_to_replace, new_ld_flags),
        );

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&makefile)?;
        file.write(contents.as_bytes())?;

        std::fs::copy(
            &makefile,
            makefile_directory.join("Makefile.win32.common.fixed"),
        )?;

        Ok(())
    }

    fn patch_windows_features_makefile(
        &self,
        options: &BundleOptions,
    ) -> Result<(), Box<dyn Error>> {
        let makefile_directory = self.source_directory(options).join("build");

        if makefile_directory
            .join("Makefile.win32.features-h.fixed")
            .exists()
        {
            return Ok(());
        }

        let makefile = makefile_directory.join("Makefile.win32.features-h");

        std::fs::copy(
            &makefile,
            makefile_directory.join("Makefile.win32.features-h.bak"),
        )?;

        let mut contents = read_to_string(&makefile)?;
        contents = contents.replace("@echo", "@coreutils echo");

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&makefile)?;
        file.write(contents.as_bytes())?;

        std::fs::copy(
            &makefile,
            makefile_directory.join("Makefile.win32.features-h.fixed"),
        )?;

        Ok(())
    }

    fn patch_windows_makefile(&self, options: &BundleOptions) -> Result<(), Box<dyn Error>> {
        let makefile_directory = self.source_directory(options).join("src");

        if makefile_directory.join("Makefile.win32.fixed").exists() {
            return Ok(());
        }

        let makefile = makefile_directory.join("Makefile.win32");

        std::fs::copy(&makefile, makefile_directory.join("Makefile.win32.bak"))?;

        let mut contents = read_to_string(&makefile)?;
        contents = contents.replace(
            "@for x in $(enabled_cairo_headers); do echo \"	src/$$x\"; done",
            "",
        );

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&makefile)?;
        file.write(contents.as_bytes())?;

        std::fs::copy(&makefile, makefile_directory.join("Makefile.win32.fixed"))?;

        Ok(())
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

        if options.target().is_unix() {
            self.compile_unix(options).expect("Failed to compile cairo")
        }
        if options.target().is_windows() {
            self.compile_windows(options)
                .expect("Failed to compile cairo")
        }
    }

    fn compiled_library_directories(&self, options: &BundleOptions) -> Vec<PathBuf> {
        if options.target().is_unix() {
            let lib = self.native_library_prefix(options).join("lib");
            return vec![lib];
        }
        if options.target().is_windows() {
            let lib = self
                .native_library_prefix(options)
                .join("src")
                .join(options.profile());
            return vec![lib];
        }
        vec![]
    }

    fn has_dependencies(&self, _options: &BundleOptions) -> bool {
        !self.dependencies.is_empty()
    }

    fn ensure_requirements(&self, options: &BundleOptions) {
        which::which("make").expect("Could not find `make`");
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

impl NativeLibrary for CairoLibrary {
    fn native_library_prefix(&self, options: &BundleOptions) -> PathBuf {
        if options.target().is_windows() {
            return self.source_directory(options);
        }

        options.target_dir().join(self.name())
    }

    fn native_library_dependency_prefixes(&self, options: &BundleOptions) -> Vec<PathBuf> {
        self.dependencies.dependency_prefixes(options)
    }

    fn clone_native_library(&self) -> Box<dyn NativeLibrary> {
        Box::new(Clone::clone(self))
    }
}

impl From<CairoLibrary> for Box<dyn Library> {
    fn from(library: CairoLibrary) -> Self {
        Box::new(library)
    }
}
