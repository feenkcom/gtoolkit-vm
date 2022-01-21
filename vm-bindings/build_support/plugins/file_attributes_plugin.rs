#[cfg(not(feature = "file_attributes_plugin"))]
compile_error!("file_attributes_plugin must be enabled for this crate.");

#[cfg(not(feature = "file_plugin"))]
compile_error!("file_plugin must be enabled for this crate.");

use crate::{file_plugin, Core, Dependency, Plugin};
use crate::build_support::CompilationUnit;

pub fn file_attributes_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("FileAttributesPlugin", core);
    plugin.dependency(Dependency::Plugin(file_plugin(core)));
    if plugin.target().is_windows() {
        plugin.include("{sources}/extracted/plugins/FilePlugin/include/win");
    }
    plugin
}
