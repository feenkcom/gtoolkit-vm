#[cfg(not(feature = "large_integers_plugin"))]
compile_error!("large_integers_plugin must be enabled for this crate.");

use crate::{Core, Plugin};

pub fn large_integers_plugin(core: &Core) -> Option<Plugin> {
    Plugin::extracted("LargeIntegers", core).into()
}
