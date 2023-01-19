use crate::{CompilationUnit, Core, Dependency, Plugin};

#[cfg(not(feature = "locale_plugin"))]
compile_error!("locale_plugin must be enabled for this crate.");

pub fn locale_plugin(core: &Core) -> Option<Plugin> {
    let mut plugin = Plugin::extracted("LocalePlugin", core);
    if plugin.family().is_apple() {
        plugin.dependency(Dependency::SystemLibrary("CoreFoundation".to_string()));
    }
    plugin.into()
}
