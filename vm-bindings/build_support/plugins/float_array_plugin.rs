#[cfg(not(feature = "float_array_plugin"))]
compile_error!("float_array_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Plugin};

pub fn float_array_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::extracted("FloatArrayPlugin", core);
    plugin.source("{generated}/plugins/src/FloatArrayPlugin/FloatArrayPlugin.c");
    plugin
}
