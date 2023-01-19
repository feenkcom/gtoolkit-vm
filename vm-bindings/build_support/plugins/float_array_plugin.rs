#[cfg(not(feature = "float_array_plugin"))]
compile_error!("float_array_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Plugin};

pub fn float_array_plugin(core: &Core) -> Option<Plugin> {
    if !core
        .builder()
        .generated_directory()
        .join("plugins/src/FloatArrayPlugin")
        .exists()
    {
        return None;
    }
    let mut plugin = Plugin::extracted("FloatArrayPlugin", core);
    plugin.source("{generated}/plugins/src/FloatArrayPlugin/FloatArrayPlugin.c");
    plugin.into()
}
