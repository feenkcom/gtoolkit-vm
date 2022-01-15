use crate::{Builder, BuilderTarget, CompilationUnit, Core, Dependency, Plugin};
use file_matcher::{FileNamed, FilesNamed};

pub fn file_plugin(core: Core) -> Plugin {
    let mut file_plugin = Plugin::new("FilePlugin", core);
    file_plugin.add_includes();

    file_plugin.sources("{generated}/64/plugins/src/FilePlugin/FilePlugin.c");
    match file_plugin.builder().target() {
        BuilderTarget::MacOS => {
            file_plugin.sources("{sources}/extracted/plugins/FilePlugin/src/common/*.c");
            file_plugin.sources("{sources}/extracted/plugins/FilePlugin/src/osx/*.c");
            file_plugin.sources("{sources}/extracted/vm/src/unix/sqUnixCharConv.c");
            file_plugin.sources("{sources}/src/fileUtils.c");
        }
        BuilderTarget::Linux => {
            file_plugin.sources("{sources}/extracted/plugins/FilePlugin/src/common/*.c");
            file_plugin.sources("{sources}/extracted/plugins/FilePlugin/src/unix/*.c");
            file_plugin.sources("{sources}/extracted/vm/src/unix/sqUnixCharConv.c");
            file_plugin.sources("{sources}/src/fileUtils.c");
        }
        BuilderTarget::Windows => {
            file_plugin.sources("{sources}/extracted/plugins/FilePlugin/src/win/*.c");
            file_plugin.sources("{sources}/extracted/vm/src/sqWin32Directory.c");
            file_plugin.sources("{sources}/src/fileUtilsWin.c");
        }
    }
    file_plugin.add_dependency(Dependency::Framework("AppKit".to_string()));

    file_plugin
}
