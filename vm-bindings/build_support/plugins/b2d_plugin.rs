#[cfg(not(feature = "b2d_plugin"))]
compile_error!("b2d_plugin must be enabled for this crate.");

use crate::{Core, Plugin};

pub fn b2d_plugin(core: &Core) -> Option<Plugin> {
    Plugin::extracted("B2DPlugin", core).into()
}
