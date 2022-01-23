#[cfg(not(feature = "uuid_plugin"))]
compile_error!("uuid_plugin must be enabled for this crate.");

use crate::{BuilderTarget, CompilationUnit, Core, Dependency, Plugin};

pub fn uuid_plugin(core: &Core) -> Option<Plugin> {
    let mut plugin = Plugin::new("UUIDPlugin", core);

    plugin.define_for_header("sys/uuid.h", "HAVE_SYS_UUID_H");
    plugin.define_for_header("uuid/uuid.h", "HAVE_UUID_UUID_H");
    plugin.define_for_header("uuid.h", "HAVE_UUID_H");

    plugin.source("{sources}/plugins/UUIDPlugin/common/UUIDPlugin.c");
    match plugin.builder().target() {
        BuilderTarget::MacOS => {}
        BuilderTarget::Linux => {
            let uuid_lib = pkg_config::probe_library("uuid").unwrap();
            plugin.add_includes(uuid_lib.include_paths);
            plugin.dependency(Dependency::Library("uuid".to_string(), vec![]));
        }
        BuilderTarget::Windows => {
            plugin.dependency(Dependency::SystemLibrary("ole32".to_string()));
        }
    }

    plugin.into()
}
