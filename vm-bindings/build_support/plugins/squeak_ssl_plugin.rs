#[cfg(not(feature = "squeak_ssl_plugin"))]
compile_error!("squeak_ssl_plugin must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Plugin};

pub fn squeak_ssl_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("SqueakSSL", core);
    match plugin.target() {
        BuilderTarget::MacOS => {
            plugin.dependency(Dependency::SystemLibrary("CoreFoundation".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Security".to_string()));
        }
        BuilderTarget::Linux => {
            let openssl = pkg_config::Config::new()
                .cargo_metadata(false)
                .probe("openssl")
                .unwrap();
            for lib in &openssl.libs {
                plugin.dependency(Dependency::Library(
                    lib.to_string(),
                    openssl.link_paths.clone(),
                ));
            }
            plugin.add_includes(&openssl.include_paths);
        }
        BuilderTarget::Windows => {
            plugin.dependency(Dependency::SystemLibrary("Crypt32".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Secur32".to_string()));
        }
    }
    plugin
}
