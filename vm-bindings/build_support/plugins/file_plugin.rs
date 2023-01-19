#[cfg(not(feature = "file_plugin"))]
compile_error!("file_plugin must be enabled for this crate.");

use crate::{CompilationUnit, Core, Dependency, Plugin, FamilyOS};

pub fn file_plugin(core: &Core) -> Option<Plugin> {
    let target = core.target();

    let mut file_plugin = Plugin::new("FilePlugin", core);
    file_plugin.with_default_includes();
    file_plugin.source("{generated}/plugins/src/FilePlugin/FilePlugin.c");

    match target.family() {
        FamilyOS::Apple => {
            file_plugin.source("{sources}/extracted/plugins/FilePlugin/src/common/*.c");
            file_plugin.source("{sources}/extracted/plugins/FilePlugin/src/osx/*.c");
            file_plugin.source("{sources}/extracted/vm/src/unix/sqUnixCharConv.c");
            file_plugin.source("{sources}/src/fileUtils.c");
            file_plugin.dependency(Dependency::SystemLibrary("Foundation".to_string()));
        }
        FamilyOS::Unix | FamilyOS::Other => {
            file_plugin.source("{sources}/extracted/plugins/FilePlugin/src/common/*.c");
            file_plugin.source("{sources}/extracted/plugins/FilePlugin/src/unix/*.c");
            file_plugin.source("{sources}/extracted/vm/src/unix/sqUnixCharConv.c");
            file_plugin.source("{sources}/src/fileUtils.c");
        }
        FamilyOS::Windows => {
            file_plugin.source("{sources}/extracted/plugins/FilePlugin/src/win/*.c");
            file_plugin.source("{sources}/extracted/vm/src/win/sqWin32Directory.c");
            file_plugin.source("{sources}/src/fileUtilsWin.c");
            file_plugin.define("WIN32_FILE_SUPPORT", None);
        }
    }

    file_plugin.into()
}
