use crate::{Builder, Core, Plugin};

pub fn misc_primitive_plugin(core: Core) -> Plugin {
    Plugin::extracted("MiscPrimitivePlugin", core)
}
