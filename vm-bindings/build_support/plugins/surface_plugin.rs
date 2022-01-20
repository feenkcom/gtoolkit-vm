#[cfg(not(feature = "surface_plugin"))]
compile_error!("socket_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Plugin};

pub fn surface_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("SurfacePlugin", core);
    plugin.source("{generated}/plugins/src/SurfacePlugin/SurfacePlugin.c");
    plugin
}
