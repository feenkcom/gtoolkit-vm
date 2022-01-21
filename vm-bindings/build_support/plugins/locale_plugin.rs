#[cfg(not(feature = "locale_plugin"))]
compile_error!("locale_plugin must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Plugin};

pub fn locale_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("LocalePlugin", core);
    match plugin.target() {
        BuilderTarget::MacOS => {
            plugin.dependency(Dependency::SystemLibrary("CoreFoundation".to_string()));
        }
        BuilderTarget::Linux => {}
        BuilderTarget::Windows => {}
    }
    plugin
}
