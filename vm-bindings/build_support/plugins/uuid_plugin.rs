#[cfg(not(feature = "uuid_plugin"))]
compile_error!("uuid_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Dependency, FamilyOS, Plugin};

pub fn uuid_plugin(core: &Core) -> Option<Plugin> {
    if core.target().is_android() {
        return None;
    }
    let mut plugin = Plugin::new("UUIDPlugin", core);

    plugin.define_for_header("sys/uuid.h", "HAVE_SYS_UUID_H");
    plugin.define_for_header("uuid/uuid.h", "HAVE_UUID_UUID_H");
    plugin.define_for_header("uuid.h", "HAVE_UUID_H");

    plugin.source("{sources}/plugins/UUIDPlugin/common/UUIDPlugin.c");
    match plugin.builder().target_family() {
        FamilyOS::Apple => {}
        FamilyOS::Unix => {
            let uuid_lib = pkg_config::probe_library("uuid").unwrap();
            let includes: Vec<String> = uuid_lib
                .include_paths
                .iter()
                .map(|each| each.display().to_string())
                .collect();
            plugin.add_includes(includes);
            plugin.dependency(Dependency::Library("uuid".to_string(), vec![]));
        }
        FamilyOS::Windows => {
            plugin.dependency(Dependency::SystemLibrary("ole32".to_string()));
        }
        FamilyOS::Other => {}
    }

    plugin.into()
}
