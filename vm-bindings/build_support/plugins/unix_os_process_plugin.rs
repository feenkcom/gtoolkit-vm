#[cfg(all(not(feature = "unix_os_process_plugin"), target_family = "unix"))]
compile_error!("unix_os_process_plugin must be enabled for this crate.");

#[cfg(not(feature = "socket_plugin"))]
compile_error!("socket_plugin must be enabled for this crate.");

#[cfg(not(feature = "file_plugin"))]
compile_error!("file_plugin must be enabled for this crate.");

use crate::{file_plugin, socket_plugin, CompilationUnit, Core, Dependency, Plugin};

pub fn unix_os_process_plugin(core: &Core) -> Plugin {
    let file_plugin = file_plugin(core);
    let socket_plugin = socket_plugin(core);

    let mut plugin = Plugin::extracted("UnixOSProcessPlugin", core);
    plugin.add_includes(file_plugin.get_includes());
    plugin.add_includes(socket_plugin.get_includes());
    plugin.dependency(Dependency::Plugin(file_plugin));
    plugin.dependency(Dependency::Plugin(socket_plugin));
    plugin
}
