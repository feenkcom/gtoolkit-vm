use libopenssl_library::libopenssl;
use shared_library_builder::{Library, LibraryCompilationContext, LibraryTarget};

use crate::{CompilationUnit, Core, Dependency, FamilyOS, Plugin};

#[cfg(not(feature = "squeak_ssl_plugin"))]
compile_error!("squeak_ssl_plugin must be enabled for this crate.");

pub fn squeak_ssl_plugin(core: &Core) -> Option<Plugin> {
    let mut plugin = Plugin::extracted("SqueakSSL", core);
    match plugin.family() {
        FamilyOS::Apple => {
            plugin.dependency(Dependency::SystemLibrary("CoreFoundation".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Security".to_string()));
        }
        FamilyOS::Unix => {
            let library_target =
                LibraryTarget::try_from(core.builder().platform().to_string().as_str()).unwrap();
            let debug = match std::env::var("PROFILE").unwrap().as_str() {
                "debug" => true,
                _ => false,
            };
            let src_dir = core.output_directory().join("openssl");
            if !src_dir.exists() {
                std::fs::create_dir_all(&src_dir).expect("Create scr dir");
            }
            let build_dir = core.output_directory().join("openssl-build");
            if !build_dir.exists() {
                std::fs::create_dir_all(&build_dir).expect("Create build dir");
            }

            let marker_file = build_dir.join("compiled.marker");
            if !marker_file.exists() {
                let context =
                    LibraryCompilationContext::new(&src_dir, &build_dir, library_target, debug);

                let openssl_version: Option<String> = None;

                let mut openssl = libopenssl(openssl_version);
                openssl.be_static();

                let ssl = openssl.be_ssl();
                ssl.just_compile(&context).expect("Failed to build openssl");
                std::fs::File::create(marker_file.as_path()).expect("Marker creation failed");
            }

            plugin.dependency(Dependency::Library(
                "ssl".to_string(),
                vec![build_dir.join("ssl").join("build").join("lib")],
            ));
            plugin.dependency(Dependency::Library(
                "crypto".to_string(),
                vec![build_dir.join("ssl").join("build").join("lib")],
            ));
            plugin.add_include("{output}/openssl-build/ssl/build/include");
        }
        FamilyOS::Windows => {
            plugin.dependency(Dependency::SystemLibrary("Crypt32".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Secur32".to_string()));
        }
        FamilyOS::Other => {}
    }
    plugin.into()
}
