use crate::{file_plugin, Builder, Core, Dependency, Plugin};

pub fn file_attributes_plugin(core: Core) -> Plugin {
    let mut plugin = Plugin::extracted("FileAttributesPlugin", core.clone());
    plugin.add_dependency(Dependency::Plugin(file_plugin(core.clone())));
    plugin
}
