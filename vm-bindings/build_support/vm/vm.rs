use anyhow::{anyhow, Result};
use std::ffi::c_void;
use std::mem;
use std::os::raw::{c_int, c_long, c_longlong};
use std::rc::Rc;

use crate::*;

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    builder: Rc<dyn Builder>,
    vmmaker: VMMaker,
    build_info: BuildInfo,
    config: ConfigTemplate,
    core: Core,
    plugins: Vec<Plugin>,
}

impl VirtualMachine {
    fn builder() -> Result<Rc<dyn Builder>> {
        match std::env::consts::OS {
            "linux" => Ok(LinuxBuilder::default().boxed()),
            "macos" => Ok(MacBuilder::default().boxed()),
            "windows" => Ok(WindowsBuilder::default().boxed()),
            _ => Err(anyhow!(
                "The platform you're compiling for is not supported"
            )),
        }
    }

    fn build_info(builder: Rc<dyn Builder>) -> Result<BuildInfo> {
        BuildInfo::new(builder)
    }

    /// Sets up the core configuration of the vm such as its name, the size of basic types, version and the build timestamp
    fn config(builder: Rc<dyn Builder>, info: &BuildInfo) -> Result<ConfigTemplate> {
        let mut config = ConfigTemplate::new(builder.clone());

        let size_of_int = mem::size_of::<c_int>();
        let size_of_long = mem::size_of::<c_long>();
        let size_of_long_long = mem::size_of::<c_longlong>();
        let size_of_void_p = mem::size_of::<*const c_void>();
        let squeak_int64_type = if size_of_long == 8 {
            "long"
        } else {
            if size_of_long_long == 8 {
                "long long"
            } else {
                return Err(anyhow!("Could not find a 64bit integer type"));
            }
        };

        let os_type = match builder.target() {
            BuilderTarget::MacOS => "Mac OS",
            BuilderTarget::Linux => "unix",
            BuilderTarget::Windows => "Win32",
        };

        let target_os = match builder.target() {
            BuilderTarget::MacOS => "1000",
            BuilderTarget::Linux => "linux-gnu",
            BuilderTarget::Windows => "Win64",
        };

        config
            .var(Config::VM_NAME("Pharo".to_string()))
            .var(Config::DEFAULT_IMAGE_NAME("Pharo.image".to_string()))
            .var(Config::OS_TYPE(os_type.to_string()))
            .var(Config::VM_TARGET(std::env::var("CARGO_CFG_TARGET_OS")?))
            .var(Config::VM_TARGET_OS(target_os.to_string()))
            .var(Config::VM_TARGET_CPU(std::env::var(
                "CARGO_CFG_TARGET_ARCH",
            )?))
            .var(Config::SIZEOF_INT(size_of_int))
            .var(Config::SIZEOF_LONG(size_of_long))
            .var(Config::SIZEOF_LONG_LONG(size_of_long_long))
            .var(Config::SIZEOF_VOID_P(size_of_void_p))
            .var(Config::SQUEAK_INT64_TYPEDEF(squeak_int64_type.to_string()))
            .var(Config::VERSION_MAJOR(info.version_major()))
            .var(Config::VERSION_MINOR(info.version_minor()))
            .var(Config::VERSION_PATCH(info.version_patch()))
            .var(Config::BUILT_FROM(info.to_string()))
            .var(Config::ALWAYS_INTERACTIVE(false));
        Ok(config)
    }

    fn vmmaker(builder: Rc<dyn Builder>) -> Result<VMMaker> {
        let vmmaker = VMMaker::prepare(builder)?;
        vmmaker.generate_sources();
        Ok(vmmaker)
    }

    fn sources(target: &BuilderTarget) -> Vec<&str> {
        let mut sources = [
            // generated interpreter sources
            "{generated}/vm/src/cogit.c",
            #[cfg(not(feature = "gnuisation"))]
            "{generated}/vm/src/cointerp.c",
            #[cfg(feature = "gnuisation")]
            "{generated}/vm/src/gcc3x-cointerp.c",
            // support sources
            "{sources}/src/debug.c",
            "{sources}/src/utils.c",
            "{sources}/src/errorCode.c",
            "{sources}/src/nullDisplay.c",
            "{sources}/src/externalPrimitives.c",
            "{sources}/src/client.c",
            "{sources}/src/pathUtilities.c",
            "{sources}/src/parameterVector.c",
            "{sources}/src/parameters.c",
            "{sources}/src/fileDialogCommon.c",
            "{sources}/src/stringUtilities.c",
            "{sources}/src/imageAccess.c",
            "{sources}/src/semaphores/platformSemaphore.c",
            "{sources}/extracted/vm/src/common/heartbeat.c",
            // Common sources
            "{sources}/extracted/vm/src/common/sqHeapMap.c",
            "{sources}/extracted/vm/src/common/sqVirtualMachine.c",
            "{sources}/extracted/vm/src/common/sqNamedPrims.c",
            "{sources}/extracted/vm/src/common/sqExternalSemaphores.c",
            "{sources}/extracted/vm/src/common/sqTicker.c",
        ]
        .to_vec();

        match target {
            BuilderTarget::MacOS => {
                sources.extend([
                    // Platform sources
                    "{sources}/extracted/vm/src/osx/aioOSX.c",
                    "{sources}/src/debugUnix.c",
                    "{sources}/src/utilsMac.mm",
                    // Support sources
                    "{sources}/src/fileDialogMac.m",
                    // Virtual Memory functions
                    "{sources}/src/memoryUnix.c",
                ])
            }
            BuilderTarget::Linux => {
                sources.extend([
                    // Platform sources
                    "{sources}/extracted/vm/src/unix/aio.c",
                    "{sources}/src/debugUnix.c",
                    // Support sources
                    "{sources}/src/fileDialogUnix.c",
                    // Virtual Memory functions
                    "{sources}/src/memoryUnix.c",
                ])
            }
            BuilderTarget::Windows => {
                sources.extend([
                    // Platform sources
                    "{sources}/extracted/vm/src/win/sqWin32SpurAlloc.c",
                    "{sources}/extracted/vm/src/win/aioWin.c",
                    "{sources}/src/win/winDebug.c",
                    "{sources}/src/win/winDebugMenu.c",
                    "{sources}/src/win/winDebugWindow.c",
                    // Support sources
                    "{sources}/src/fileDialogWin32.c",
                    "{sources}/src/utils/setjmp-Windows-wrapper-X64.S",
                ])
            }
        }

        sources
    }

    /// Return a list of include directories for a given build taregt platform
    fn includes(target: &BuilderTarget) -> Vec<&str> {
        let mut includes = [
            "{sources}/extracted/vm/include/common",
            "{sources}/include",
            "{sources}/include/pharovm",
            "{generated}/vm/include",
        ]
        .to_vec();

        match target {
            BuilderTarget::MacOS => {
                includes.push("{sources}/extracted/vm/include/osx");
                includes.push("{sources}/extracted/vm/include/unix");
            }
            BuilderTarget::Linux => {
                includes.push("{sources}/extracted/vm/include/unix");
            }
            BuilderTarget::Windows => {
                includes.push("{sources}/extracted/vm/include/win");
                includes.push("{ output }/pthreads/lib/x64/{ profile }");
            }
        }
        includes
    }

    fn core(builder: Rc<dyn Builder>) -> Core {
        let mut core = Core::new("PharoVMCore", builder.clone());
        core.sources(Self::sources(&core.target()));
        core.includes(Self::includes(&core.target()));

        core.define_for_header("dirent.h", "HAVE_DIRENT_H");
        core.define_for_header("features.h", "HAVE_FEATURES_H");
        core.define_for_header("unistd.h", "HAVE_UNISTD_H");
        core.define_for_header("ndir.h", "HAVE_NDIR_H");
        core.define_for_header("sys/ndir.h", "HAVE_SYS_NDIR_H");
        core.define_for_header("sys/dir.h", "HAVE_SYS_DIR_H");
        core.define_for_header("sys/filio.h", "HAVE_SYS_FILIO_H");
        core.define_for_header("sys/time.h", "HAVE_SYS_TIME_H");
        core.define_for_header("execinfo.h", "HAVE_EXECINFO_H");
        core.define_for_header("dlfcn.h", "HAVE_DLFCN_H");

        core.flag("-Wno-error=implicit-function-declaration");
        core.flag("-Wno-implicit-function-declaration");
        core.flag("-Wno-absolute-value");
        core.flag("-Wno-shift-count-overflow");
        core.flag("-Wno-int-conversion");
        core.flag("-Wno-macro-redefined");
        core.flag("-Wno-unused-value");
        core.flag("-Wno-pointer-to-int-cast");
        core.flag("-Wno-non-literal-null-conversion");
        core.flag("-Wno-conditional-type-mismatch");
        core.flag("-Wno-compare-distinct-pointer-types");
        core.flag("-Wno-incompatible-function-pointer-types");
        core.flag("-Wno-pointer-sign");
        core.flag("-Wno-unused-command-line-argument");
        core.flag("-Wno-undef-prefix");

        #[cfg(feature = "immutability")]
        core.define("IMMUTABILITY", "1");
        #[cfg(feature = "inline_memory_accessors")]
        core.define("USE_INLINE_MEMORY_ACCESSORS", "1");
        core.define("COGMTVM", "0");
        core.define("STACKVM", "0");
        core.define("PharoVM", "1");
        core.define("SPURVM", "1");
        core.define("ASYNC_FFI_QUEUE", "1");
        core.define("ARCH", "64");
        core.define("VM_LABEL(foo)", "0");
        core.define("SOURCE_PATH_SIZE", "80");

        #[cfg(not(debug_assertions))]
        {
            core.define("NDEBUG", None);
            core.define("DEBUGVM", "0");
        }

        if core.target().is_unix() {
            core.define("LSB_FIRST", "1");
            core.define("UNIX", "1");
            core.define("HAVE_TM_GMTOFF", None);
        }

        if core.target().is_macos() {
            core.define("OSX", "1");
            // In Apple Silicon machines the code zone is read-only, and requires special operations
            #[cfg(target_arch = "aarch64")]
            core.define("READ_ONLY_CODE_ZONE", "1");
            core.dependency(Dependency::SystemLibrary("AppKit".to_string()));
        }

        if core.target().is_windows() {
            core.define("WIN", "1");
            core.dependency(Dependency::SystemLibrary("User32".to_string()));
            core.dependency(Dependency::SystemLibrary("Ws2_32".to_string()));
            core.dependency(Dependency::SystemLibrary("DbgHelp".to_string()));
            core.dependency(Dependency::SystemLibrary("Ole32".to_string()));
            core.dependency(Dependency::Library(
                "pthreads".to_string(),
                vec![core
                    .output_directory()
                    .join("pthreads\\lib\\x64")
                    .join(core.builder().profile())],
            ));
        }

        #[cfg(feature = "ffi")]
        core.add_feature(ffi_feature(&core));
        #[cfg(feature = "threaded_ffi")]
        core.add_feature(threaded_ffi_feature(&core));
        core
    }

    fn plugins(core: &Core) -> Vec<Plugin> {
        [
            #[cfg(feature = "b2d_plugin")]
            b2d_plugin(&core),
            #[cfg(feature = "bit_blt_plugin")]
            bit_blt_plugin(&core),
            #[cfg(feature = "dsa_primitives_plugin")]
            dsa_primitives_plugin(&core),
            #[cfg(feature = "file_plugin")]
            file_plugin(&core),
            #[cfg(feature = "file_attributes_plugin")]
            file_attributes_plugin(&core),
            #[cfg(feature = "float_array_plugin")]
            float_array_plugin(&core),
            #[cfg(feature = "jpeg_read_writer2_plugin")]
            jpeg_read_writer2_plugin(&core),
            #[cfg(feature = "jpeg_reader_plugin")]
            jpeg_reader_plugin(&core),
            #[cfg(feature = "large_integers_plugin")]
            large_integers_plugin(&core),
            #[cfg(feature = "locale_plugin")]
            locale_plugin(&core),
            #[cfg(feature = "misc_primitive_plugin")]
            misc_primitive_plugin(&core),
            #[cfg(feature = "socket_plugin")]
            socket_plugin(&core),
            #[cfg(feature = "squeak_ssl_plugin")]
            squeak_ssl_plugin(&core),
            #[cfg(feature = "surface_plugin")]
            surface_plugin(&core),
            #[cfg(all(feature = "unix_os_process_plugin", target_family = "unix"))]
            unix_os_process_plugin(&core),
            #[cfg(feature = "uuid_plugin")]
            uuid_plugin(&core),
        ]
        .to_vec()
        .into_iter()
        .filter_map(|each| each)
        .collect()
    }

    pub fn new() -> Result<Self> {
        let builder = Self::builder()?;
        builder.prepare_environment();
        let build_info = Self::build_info(builder.clone())?;
        let config = Self::config(builder.clone(), &build_info)?;
        let vmmaker = Self::vmmaker(builder.clone())?;
        let core = Self::core(builder.clone());
        let plugins = Self::plugins(&core);

        Ok(Self {
            builder,
            vmmaker,
            build_info,
            config,
            core,
            plugins,
        })
    }

    pub fn compile(&self) {
        self.config.render();
        self.core.compile();
        for plugin in &self.plugins {
            plugin.compile();
        }
        self.builder.link_libraries();
        self.builder.generate_bindings();
    }
}
