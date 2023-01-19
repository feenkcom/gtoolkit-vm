#[cfg(not(feature = "squeak_ssl_plugin"))]
compile_error!("squeak_ssl_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Dependency, FamilyOS, Plugin};

pub fn squeak_ssl_plugin(core: &Core) -> Option<Plugin> {
    let mut plugin = Plugin::extracted("SqueakSSL", core);
    match plugin.family() {
        FamilyOS::Apple => {
            plugin.dependency(Dependency::SystemLibrary("CoreFoundation".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Security".to_string()));
        }
        FamilyOS::Unix => {
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
            let includes: Vec<String> = openssl
                .include_paths
                .iter()
                .map(|each| each.display().to_string())
                .collect();
            plugin.add_includes(includes);
        }
        FamilyOS::Windows => {
            plugin.dependency(Dependency::SystemLibrary("Crypt32".to_string()));
            plugin.dependency(Dependency::SystemLibrary("Secur32".to_string()));
        }
        FamilyOS::Other => {}
    }
    plugin.into()
}
