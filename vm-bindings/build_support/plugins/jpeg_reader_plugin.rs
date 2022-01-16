#[cfg(not(feature = "jpeg_reader_plugin"))]
compile_error!("jpeg_reader_plugin must be enabled for this crate.");

use crate::{Core, Plugin};

pub fn jpeg_reader_plugin(core: &Core) -> Plugin {
    Plugin::extracted("JPEGReaderPlugin", core)
}
