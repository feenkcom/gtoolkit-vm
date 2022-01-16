#[cfg(not(feature = "jpeg_read_writer2_plugin"))]
compile_error!("jpeg_read_writer2_plugin must be enabled for this crate.");

use crate::{Core, Plugin};

pub fn jpeg_read_writer2_plugin(core: &Core) -> Plugin {
    Plugin::extracted("JPEGReadWriter2Plugin", core)
}
