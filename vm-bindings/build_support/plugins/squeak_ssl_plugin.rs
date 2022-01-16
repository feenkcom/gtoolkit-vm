#[cfg(not(feature = "squeak_ssl_plugin"))]
compile_error!("squeak_ssl_plugin must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Plugin};

pub fn squeak_ssl_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("SqueakSSL", core);
    match plugin.target() {
        BuilderTarget::MacOS => {
            plugin.add_dependency(Dependency::Framework("CoreFoundation".to_string()));
            plugin.add_dependency(Dependency::Framework("Security".to_string()));
        }
        BuilderTarget::Linux => {
            let openssl = pkg_config::probe_library("OpenSSL").unwrap();
            for lib in &openssl.libs {
                plugin.add_dependency(Dependency::Library(lib.to_string()));
            }
        }
        BuilderTarget::Windows => {
            plugin.add_dependency(Dependency::Library("Crypt32".to_string()));
            plugin.add_dependency(Dependency::Library("Secur32".to_string()));
        }
    }
    plugin
}
