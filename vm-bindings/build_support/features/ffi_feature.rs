use std::process::Command;
use std::{
    env,
    path::{Path, PathBuf},
};

use clang_sys::support::Clang;

use crate::{CompilationUnit, Core, Dependency, FamilyOS, Feature, WindowsBuilder};

#[cfg(not(feature = "ffi"))]
compile_error!("ffi must be enabled for this crate.");

pub fn ffi_feature(core: &Core) -> Feature {
    let mut feature = Feature::new("FFI", core);
    feature.define("FEATURE_FFI", "1");
    feature.include("{sources}/ffi/include");

    feature.source("{sources}/ffi/src/functionDefinitionPrimitives.c");
    feature.source("{sources}/ffi/src/primitiveCalls.c");
    feature.source("{sources}/ffi/src/primitiveUtils.c");
    feature.source("{sources}/ffi/src/types.c");
    feature.source("{sources}/ffi/src/typesPrimitives.c");
    feature.source("{sources}/ffi/src/utils.c");
    // Single-threaded callout support
    feature.source("{sources}/ffi/src/sameThread/sameThread.c");
    // Callback support
    feature.source("{sources}/ffi/src/callbacks/callbackPrimitives.c");
    feature.source("{sources}/ffi/src/callbacks/callbacks.c");
    // Required by callbacks
    feature.source("{sources}/src/semaphores/pharoSemaphore.c");
    feature.source("{sources}/src/threadSafeQueue/threadSafeQueue.c");

    match core.family() {
        FamilyOS::Unix => {
            compile_and_include_ffi(core, &mut feature);
        }
        FamilyOS::Apple => {
            if core.target().is_macos() && cfg!(target_arch = "aarch64") {
                include_system_ffi(&mut feature);
            }
            // only MacOS allows developers to include ffi headers from the SDK.
            else {
                compile_and_include_ffi(core, &mut feature);
            }
        }
        FamilyOS::Windows => {
            let lib_ffi = WindowsBuilder::install_ffi().join("lib");
            feature.include(format!(
                "{{ output }}/{}/{}/include",
                WindowsBuilder::ffi_name(),
                WindowsBuilder::vcpkg_triplet()
            ));
            feature.dependency(Dependency::Library("libffi".to_string(), vec![lib_ffi]));
        }
        FamilyOS::Other => {
            panic!("FFI is not available on {}", core.target())
        }
    }

    feature
}

fn compile_and_include_ffi(core: &Core, feature: &mut Feature) {
    build_and_link(core, feature).expect("Failed to compile ffi");
}

fn include_system_ffi(feature: &mut Feature) {
    let clang = Clang::find(None, &[]).unwrap();
    let mut ffi_includes = vec![];
    if let Some(c_search_paths) = clang.c_search_paths {
        for search_path in &c_search_paths {
            if search_path.join("ffi").join("ffi.h").exists() {
                ffi_includes.push(search_path.clone().display().to_string());
                ffi_includes.push(search_path.join("ffi").display().to_string());
            }
        }
    }
    feature.add_includes(ffi_includes);
    feature.dependency(Dependency::Library("ffi".to_string(), vec![]));
}

fn run_command(which: &'static str, cmd: &mut Command) {
    assert!(cmd.status().expect(which).success(), "{}", which);
}

fn build_and_link(core: &Core, feature: &mut Feature) -> anyhow::Result<()> {
    let ffi_sources = core.output_directory().join("libffi");
    if !ffi_sources.exists() {
        Command::new("git")
            .current_dir(core.output_directory())
            .arg("clone")
            .arg("https://github.com/libffi/libffi.git")
            .arg("--branch")
            .arg("v3.4.4")
            .status()?;
    }

    let out_dir = core.output_directory();
    let build_dir = Path::new(&out_dir).join("libffi");
    let prefix = Path::new(&out_dir).join("libffi-root");

    // Generate configure, run configure, make, make install
    configure_libffi(prefix.clone(), &build_dir);

    run_command(
        "Building libffi",
        Command::new("make")
            .env_remove("DESTDIR")
            .arg("install")
            .current_dir(&build_dir),
    );

    feature.include("{output}/libffi-root/include");
    feature.dependency(Dependency::Library(
        "ffi".to_string(),
        vec![prefix.join("lib")],
    ));

    Ok(())
}

pub fn configure_libffi(prefix: PathBuf, build_dir: &Path) {
    if !build_dir.join("configure").exists() {
        run_command(
            "Create configure",
            Command::new("sh").arg("autogen.sh").current_dir(build_dir),
        );
    }

    let mut command = Command::new("sh");

    command
        .arg("configure")
        .arg("--with-pic")
        .arg("--disable-shared")
        .arg("--disable-docs");

    let target = std::env::var("TARGET").unwrap();
    let host = std::env::var("HOST").unwrap();
    if target != host {
        let cross_host = match target.as_str() {
            // Autoconf uses riscv64 while Rust uses riscv64gc for the architecture
            "riscv64gc-unknown-linux-gnu" => "riscv64-unknown-linux-gnu",
            // Autoconf does not yet recognize illumos, but Solaris should be fine
            "x86_64-unknown-illumos" => "x86_64-unknown-solaris",
            // The sources for `ios-sim` should be the same as `ios`.
            "aarch64-apple-ios-sim" => "aarch64-apple-ios",
            // Everything else should be fine to pass straight through
            other => other,
        };
        command.arg(format!("--host={}", cross_host));
    }

    let mut c_cfg = cc::Build::new();
    c_cfg
        .cargo_metadata(false)
        .target(&target)
        .warnings(false)
        .host(&host);
    let c_compiler = c_cfg.get_compiler();

    command.env("CC", c_compiler.path());

    let mut cflags = c_compiler.cflags_env();
    match env::var_os("CFLAGS") {
        None => (),
        Some(flags) => {
            cflags.push(" ");
            cflags.push(&flags);
        }
    }
    command.env("CFLAGS", cflags);

    for (k, v) in c_compiler.env().iter() {
        command.env(k, v);
    }

    command.current_dir(&build_dir);

    if cfg!(windows) {
        // When using MSYS2, OUT_DIR will be a Windows like path such as
        // C:\foo\bar. Unfortunately, the various scripts used for building
        // libffi do not like such a path, so we have to turn this into a Unix
        // like path such as /c/foo/bar.
        //
        // This code assumes the path only uses : for the drive letter, and only
        // uses \ as a component separator. It will likely break for file paths
        // that include a :.
        let mut msys_prefix = prefix
            .to_str()
            .unwrap()
            .replace(":\\", "/")
            .replace("\\", "/");

        msys_prefix.insert(0, '/');

        command.arg("--prefix").arg(msys_prefix);
    } else {
        command.arg("--prefix").arg(prefix);
    }
    run_command("Configuring libffi", &mut command);
}
