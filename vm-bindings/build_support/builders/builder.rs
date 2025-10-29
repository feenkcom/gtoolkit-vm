use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fmt};

use bindgen::CargoCallbacks;
use platforms::target::OS;
use platforms::{Platform, PointerWidth};

pub const SOURCES_DIRECTORY: &str = "pharo-vm";

#[derive(Debug, Clone, PartialEq)]
pub struct OperatingSystem<T>(OS, PhantomData<T>);
#[derive(Debug, Clone, PartialEq)]
pub struct HostType;
#[derive(Debug, Clone, PartialEq)]
pub struct TargetType;

pub type TargetOS = OperatingSystem<TargetType>;
pub type HostOS = OperatingSystem<HostType>;

#[allow(dead_code)]
impl<T> OperatingSystem<T> {
    pub fn family(&self) -> FamilyOS {
        FamilyOS::from(self)
    }

    pub fn os(&self) -> &OS {
        &self.0
    }

    pub fn is_unix(&self) -> bool {
        FamilyOS::from(self).is_unix()
    }
    pub fn is_windows(&self) -> bool {
        FamilyOS::from(self).is_windows()
    }
    pub fn is_apple(&self) -> bool {
        FamilyOS::from(self).is_apple()
    }
    pub fn is_ios(&self) -> bool {
        self.0 == OS::iOS
    }
    pub fn is_macos(&self) -> bool {
        self.0 == OS::MacOS
    }
    pub fn is_emscripten(&self) -> bool {
        self.0 == OS::Emscripten
    }
    pub fn is_android(&self) -> bool {
        self.0 == OS::Android
    }
}

impl<T> From<Platform> for OperatingSystem<T> {
    fn from(value: Platform) -> Self {
        Self(value.target_os, Default::default())
    }
}

impl<T> From<&Platform> for OperatingSystem<T> {
    fn from(value: &Platform) -> Self {
        Self(value.target_os, Default::default())
    }
}

impl<T> Display for OperatingSystem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FamilyOS {
    Unix,
    Apple,
    Windows,
    Other,
}

impl FamilyOS {
    pub fn is_unix(&self) -> bool {
        match &self {
            FamilyOS::Unix | FamilyOS::Apple => true,
            _ => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        self == &FamilyOS::Windows
    }
    pub fn is_apple(&self) -> bool {
        self == &FamilyOS::Apple
    }
}

impl<T> From<OperatingSystem<T>> for FamilyOS {
    fn from(os: OperatingSystem<T>) -> Self {
        FamilyOS::from(&os)
    }
}

impl<T> From<&OperatingSystem<T>> for FamilyOS {
    fn from(os: &OperatingSystem<T>) -> Self {
        match os.0 {
            // unix like family
            OS::Android => FamilyOS::Unix,
            OS::Emscripten => FamilyOS::Unix,
            OS::FreeBSD => FamilyOS::Unix,
            OS::Fuchsia => FamilyOS::Unix,
            OS::Linux => FamilyOS::Unix,
            OS::NetBSD => FamilyOS::Unix,
            OS::OpenBSD => FamilyOS::Unix,
            OS::Wasi => FamilyOS::Unix,
            // apple-like operating systems
            OS::iOS => FamilyOS::Apple,
            OS::MacOS => FamilyOS::Apple,
            OS::TvOS => FamilyOS::Apple,
            OS::WatchOS => FamilyOS::Apple,
            // windows-like operating systems
            OS::Windows => FamilyOS::Windows,
            // all other ones that we do not directly support.
            // we will still start a build but can not guarantee it will succeed
            OS::Cuda => FamilyOS::Other,
            OS::Dragonfly => FamilyOS::Other,
            OS::Espidf => FamilyOS::Other,
            OS::Haiku => FamilyOS::Other,
            OS::Hermit => FamilyOS::Other,
            OS::Horizon => FamilyOS::Other,
            OS::IllumOS => FamilyOS::Other,
            OS::L4re => FamilyOS::Other,
            OS::None => FamilyOS::Other,
            OS::Psp => FamilyOS::Other,
            OS::Redox => FamilyOS::Other,
            OS::Solaris => FamilyOS::Other,
            OS::SolidAsp3 => FamilyOS::Other,
            OS::Uefi => FamilyOS::Other,
            OS::Unknown => FamilyOS::Other,
            OS::VxWorks => FamilyOS::Other,
            _ => FamilyOS::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchBits {
    Bit32,
    Bit64,
}

impl From<&Platform> for ArchBits {
    fn from(value: &Platform) -> Self {
        fn unsupported(width: &PointerWidth) -> ArchBits {
            panic!("Unsupported architecture: {} bit", width);
        }

        let pointer_width = &value.target_pointer_width;

        match pointer_width {
            PointerWidth::U16 => unsupported(pointer_width),
            PointerWidth::U32 => ArchBits::Bit32,
            PointerWidth::U64 => ArchBits::Bit64,
            _ => unsupported(pointer_width),
        }
    }
}

pub trait Builder: Debug {
    fn platform(&self) -> &Platform;

    fn target(&self) -> TargetOS {
        self.platform().into()
    }

    fn host(&self) -> HostOS {
        Platform::find(env::var("HOST").unwrap().as_str())
            .unwrap()
            .into()
    }

    fn target_family(&self) -> FamilyOS {
        self.target().family()
    }

    fn host_family(&self) -> FamilyOS {
        self.host().family()
    }

    fn arch_bits(&self) -> ArchBits {
        ArchBits::from(self.platform())
    }

    fn profile(&self) -> String {
        env::var("PROFILE").unwrap()
    }

    fn is_debug(&self) -> bool {
        self.profile() == "debug"
    }

    fn output_directory(&self) -> PathBuf {
        Path::new(env::var("OUT_DIR").unwrap().as_str()).to_path_buf()
    }

    fn image_format(&self) -> &str {
        "SpurFormat"
    }

    fn artefact_directory(&self) -> PathBuf {
        let dir = self.output_directory();
        dir.parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn vm_sources_directory(&self) -> PathBuf {
        self.crate_directory().join(SOURCES_DIRECTORY)
    }

    fn crate_directory(&self) -> PathBuf {
        env::current_dir().unwrap().parent().unwrap().to_path_buf()
    }

    fn prepare_environment(&self);

    fn squeak_include_directory(&self) -> PathBuf {
        self.vm_sources_directory()
            .join("extracted")
            .join("vm")
            .join("include")
    }

    fn common_include_directory(&self) -> PathBuf {
        self.squeak_include_directory().join("common")
    }

    fn platform_include_directory(&self) -> PathBuf;

    fn generated_config_directory(&self) -> PathBuf {
        self.generated_include_directory()
    }

    fn generated_directory(&self) -> PathBuf {
        let dir = self.output_directory().join("generated");

        match self.arch_bits() {
            ArchBits::Bit32 => dir.join("32"),
            ArchBits::Bit64 => dir.join("64"),
        }
    }

    fn generated_include_directory(&self) -> PathBuf {
        self.generated_directory().join("vm").join("include")
    }

    fn generated_sources_directory(&self) -> PathBuf {
        self.generated_directory().join("vm").join("src")
    }

    fn generate_bindings(&self) {
        let include_dir = self.vm_sources_directory().join("include");

        let generated_vm_include_dir = self.generated_include_directory();
        assert!(
            generated_vm_include_dir.exists(),
            "Generated vm include directory must exist: {:?}",
            generated_vm_include_dir.display()
        );

        let generated_config_directory = self.generated_config_directory();
        assert!(
            generated_config_directory.exists(),
            "Generated config.h directory must exist: {:?}",
            generated_config_directory.display()
        );

        let extra_headers = env::current_dir().unwrap().join("extra");
        assert!(
            extra_headers.exists(),
            "Extra headers directory must exist: {:?}",
            extra_headers.display()
        );

        // cargo-apk does not configure environment for the bindgen.
        // we should configure CLANG_PATH and BINDGEN_EXTRA_CLANG_ARGS_{TARGET} variables
        if self.target().is_android() {
            let ndk = ndk_build::ndk::Ndk::from_env().unwrap();
            env::set_var("CLANG_PATH", ndk.clang().unwrap().0);
            env::set_var(
                format!("BINDGEN_EXTRA_CLANG_ARGS_{}", self.platform().target_triple),
                format!(
                    "--sysroot={}",
                    ndk.toolchain_dir().unwrap().join("sysroot").display()
                ),
            );
        }

        let mut builder = bindgen::Builder::default();
        builder = builder
            .allowlist_function("vm_.*")
            .allowlist_function("free")
            .allowlist_function("calloc")
            .allowlist_function("malloc")
            .allowlist_function("memcpy")
            .allowlist_function("registerCurrentThreadToHandleExceptions")
            .allowlist_function("installErrorHandlers")
            .allowlist_function("setProcessArguments")
            .allowlist_function("setProcessEnvironmentVector")
            .allowlist_function("getVMExports")
            .allowlist_function("setVMExports")
            // telemetry
            .allowlist_function("setTelemetry")
            .allowlist_function("takeTelemetry")
            .allowlist_function("enableTelemetry")
            .allowlist_function("disableTelemetry")
            // re-export the internal methods
            .allowlist_function("exportSqGetInterpreterProxy")
            .allowlist_function("exportOsCogStackPageHeadroom")
            .allowlist_function("exportGetHandler")
            .allowlist_function("exportReadAddress")
            .allowlist_function("exportStatFullGCUsecs")
            .allowlist_function("exportStatScavengeGCUsecs")
            .allowlist_function("exportClassOrNilAtIndex")
            .allowlist_function("exportIsOopForwarded")
            .allowlist_function("setVmRunOnWorkerThread")
            .allowlist_function("setLogger")
            .allowlist_function("setShouldLog")
            // the following functions are exported straight from the VM dynamic library,
            // rather than a VM Proxy
            .allowlist_function("createNewMethodheaderbytecodeCount")
            // InterpreterPrimitives
            .allowlist_function("primitiveFail")
            .allowlist_function("primitiveFailFor")
            // StackInterpreter
            .allowlist_function("methodReturnValue")
            .allowlist_function("methodReturnBool")
            .allowlist_function("methodReturnFloat")
            .allowlist_function("methodReturnInteger")
            .allowlist_function("methodReturnReceiver")
            .allowlist_function("methodArgumentCount")
            .allowlist_function("stackValue")
            .allowlist_function("stackFloatValue")
            .allowlist_function("stackIntegerValue")
            .allowlist_function("stackObjectValue")
            .allowlist_function("stObjectat")
            .allowlist_function("stObjectatput")
            .allowlist_function("stSizeOf")
            .allowlist_function("addressCouldBeClassObj")
            .allowlist_function("getThisContext")
            // CoInterpreter
            .allowlist_function("instVarofContext")
            // SpurMemoryManager
            .allowlist_function("falseObject")
            .allowlist_function("trueObject")
            .allowlist_function("nilObject")
            .allowlist_function("classArray")
            .allowlist_function("classExternalAddress")
            .allowlist_function("classString")
            .allowlist_function("firstIndexableField")
            .allowlist_function("firstFixedField")
            .allowlist_function("instantiateClassindexableSize")
            .allowlist_function("instantiateClassindexableSizeisPinned")
            .allowlist_function("instantiateClassisPinned")
            .allowlist_function("fetchPointerofObject")
            .allowlist_function("integerObjectOf")
            .allowlist_function("floatObjectOf")
            .allowlist_function("floatValueOf")
            .allowlist_function("isFloatInstance")
            .allowlist_function("newHashBitsOf")
            .allowlist_function("hashBitsOf")
            .allowlist_function("ensureBehaviorHash")
            .allowlist_function("firstBytePointerOfDataObject")
            .allowlist_function("isOopForwarded")
            .allowlist_function("isOld")
            .allowlist_function("isYoung")
            .allowlist_function("possibleOldObjectStoreInto")
            .allowlist_function("possiblePermObjectStoreIntovalue")
            .allowlist_function("fetchClassOfNonImm")
            .allowlist_function("stContextSize")
            .allowlist_function("isKindOfClass")
            .allowlist_function("getObjectAfterlimit")
            .allowlist_function("getOldSpaceMemoryStart")
            .allowlist_function("getOldSpaceMemoryEnd")
            .allowlist_function("getEdenSpaceMemoryStart")
            .allowlist_function("getEdenSpaceMemoryEnd")
            .allowlist_function("getPastSpaceMemoryStart")
            .allowlist_function("getPastSpaceMemoryEnd")
            .allowlist_type("sqInt")
            .allowlist_type("usqInt")
            .allowlist_type("sqExport")
            .allowlist_type("VirtualMachine")
            .header(
                include_dir
                    .join("pharovm")
                    .join("pharoClient.h")
                    .display()
                    .to_string(),
            )
            .header(
                extra_headers
                    .join("telemetry-export.h")
                    .display()
                    .to_string(),
            );

        builder = builder
            .header(extra_headers.join("sqExport.h").display().to_string())
            .header(extra_headers.join("exported.h").display().to_string())
            .header(extra_headers.join("setLogger.h").display().to_string())
            .clang_arg(format!("-I{}", &include_dir.display()))
            .clang_arg(format!("-I{}", &include_dir.join("pharovm").display()))
            .clang_arg(format!("-I{}", generated_config_directory.display()))
            .clang_arg(format!("-I{}", generated_vm_include_dir.display()))
            .clang_arg(format!("-I{}", self.common_include_directory().display()))
            .clang_arg(format!("-I{}", self.platform_include_directory().display()))
            .clang_arg("-DLSB_FIRST=1")
            // the following allows bindgen to parse headers when building for arm64 Windows
            .clang_arg("-D_NO_CRT_STDIO_INLINE")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(CargoCallbacks::new()));

        builder = builder.layout_tests(false);

        // clang is not able to generate bindings for emscripten,
        // however, since it is quite close to linux we can specify it as a target instead
        if self.target().os() == &OS::Emscripten {
            builder = builder.clang_arg(format!(
                "-I{}/upstream/emscripten/cache/sysroot/include",
                env::var("EMSDK").unwrap()
            ));
            match self.arch_bits() {
                ArchBits::Bit32 => builder = builder.clang_arg("--target=i686-unknown-linux-gnu"),
                ArchBits::Bit64 => builder = builder.clang_arg("--target=x86_64-unknown-linux-gnu"),
            }
        }

        let bindings = builder
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(self.output_directory().join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    fn link_libraries(&self) {
        println!("cargo:rustc-link-lib=PharoVMCore");
        println!(
            "cargo:rustc-link-search={}",
            self.artefact_directory().display()
        );
    }

    fn print_directories(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entry(
                &"output_directory".to_string(),
                &self.output_directory().display(),
            )
            .entry(
                &"vm_sources_directory".to_string(),
                &self.vm_sources_directory().display(),
            )
            .finish()
    }

    fn boxed(self) -> Rc<dyn Builder>;
}
