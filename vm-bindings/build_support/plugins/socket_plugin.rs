use crate::{CompilationUnit, Core, Dependency, Plugin};

#[cfg(not(feature = "socket_plugin"))]
compile_error!("socket_plugin must be enabled for this crate.");

pub fn socket_plugin(core: &Core) -> Option<Plugin> {
    let mut plugin = Plugin::extracted("SocketPlugin", core);
    if plugin.family().is_windows() {
        plugin.dependency(Dependency::SystemLibrary("Ws2_32".to_string()));
    }
    plugin.into()
}
