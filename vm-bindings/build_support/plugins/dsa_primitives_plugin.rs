#[cfg(not(feature = "dsa_primitives_plugin"))]
compile_error!("dsa_primitives_plugin must be enabled for this crate.");

use crate::{Core, Plugin};

pub fn dsa_primitives_plugin(core: &Core) -> Plugin {
    Plugin::extracted("DSAPrims", core)
}
