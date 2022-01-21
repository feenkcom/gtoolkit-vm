#[cfg(not(feature = "socket_plugin"))]
compile_error!("socket_plugin must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Plugin};

pub fn socket_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("SocketPlugin", core);
    match plugin.target() {
        BuilderTarget::MacOS => {}
        BuilderTarget::Linux => {}
        BuilderTarget::Windows => {
            plugin.dependency(Dependency::SystemLibrary("Ws2_32".to_string()));
        }
    }
    plugin
}
