#[cfg(not(feature = "bit_blt_plugin"))]
compile_error!("bit_blt_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Plugin};

pub fn bit_blt_plugin(core: &Core) -> Plugin {
    let mut plugin = Plugin::new("BitBltPlugin", core);
    plugin.with_default_includes();
    plugin.source("{sources}/extracted/plugins/BitBltPlugin/src/common/BitBltPlugin.c");
    plugin
}
