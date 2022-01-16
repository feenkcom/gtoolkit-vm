#[cfg(not(feature = "misc_primitive_plugin"))]
compile_error!("misc_primitive_plugin must be enabled for this crate.");

use crate::{Builder, Core, Plugin};

pub fn misc_primitive_plugin(core: Core) -> Plugin {
    Plugin::extracted("MiscPrimitivePlugin", core)
}
