use crate::bundlers::Bundler;
use crate::options::FinalOptions;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct WindowsBundler {}

impl WindowsBundler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_ico(&self, configuration: &FinalOptions) -> Option<PathBuf> {
        for icon in configuration.icons() {
            let icon_path = Path::new(&icon);
            if icon_path.exists() {
                if let Some(extension) = icon_path.extension() {
                    if extension == "ico" {
                        return Some(icon_path.to_path_buf());
                    }
                }
            }
        }
        None
    }

    fn temporary_directory(&self) -> PathBuf {
        std::env::current_dir().unwrap().join("temp")
    }
}

impl Bundler for WindowsBundler {
    fn pre_compile(&self, configuration: &FinalOptions) {
        let temp_dir = self.temporary_directory();

        let icon = self.create_ico(configuration);

        let info = Info {
            bundle_name: configuration.app_name(),
            bundle_identifier: self.bundle_identifier(configuration),
            bundle_author: "".to_string(),
            bundle_major_version: configuration.major_version(),
            bundle_minor_version: configuration.minor_version(),
            bundle_patch_version: configuration.patch_version(),
            bundle_icon: icon.as_ref().map_or("".to_string(), |icon| {
                format!("100 ICON \"{}\"", icon.display())
            }),
            executable_name: self.executable_name(configuration),
        };

        let resource = mustache::compile_str(RESOURCE).unwrap();
        let manifest = mustache::compile_str(MANIFEST).unwrap();

        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).unwrap();
        }

        let resource_file_path =
            temp_dir.join(format!("{}.rc", self.executable_name(configuration)));

        let manifest_file_path =
            temp_dir.join(format!("{}.manifest", self.executable_name(configuration)));

        let mut resource_file = File::create(&resource_file_path).unwrap();
        let mut manifest_file = File::create(&manifest_file_path).unwrap();

        resource.render(&mut resource_file, &info).unwrap();
        manifest.render(&mut manifest_file, &info).unwrap();

        std::env::set_var(
            "VM_CLIENT_EMBED_RESOURCES",
            format!("{}", &resource_file_path.display()),
        );
    }

    fn bundle(&self, configuration: &FinalOptions) {
        let bundle_location = configuration.bundle_location();
        let app_name = configuration.app_name();

        let app_dir = bundle_location.join(&app_name);
        let binary_dir = app_dir.join("bin");

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&binary_dir).unwrap();

        let target_executable_path = binary_dir.join(&self.executable_name(configuration));

        match fs::copy(
            self.compiled_executable_path(configuration),
            &target_executable_path,
        ) {
            Ok(_) => {}
            Err(error) => {
                panic!(
                    "Could not copy {} to {} due to {}",
                    self.compiled_executable_path(configuration).display(),
                    &target_executable_path.display(),
                    error
                );
            }
        };

        fs_extra::copy_items(
            &self.compiled_libraries(configuration),
            binary_dir,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();
    }

    fn post_compile(&self, _configuration: &FinalOptions) {
        let temp_dir = self.temporary_directory();
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
    }
}

#[derive(Serialize)]
struct Info {
    bundle_name: String,
    bundle_identifier: String,
    bundle_author: String,
    bundle_major_version: usize,
    bundle_minor_version: usize,
    bundle_patch_version: usize,
    bundle_icon: String,
    executable_name: String,
}

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <assemblyIdentity
            version="1.0.0.0"
            processorArchitecture="*"
            name="{{bundle_identifier}}"
            type="win32"
    />
    <description>Rust Manifest Example</description>
    <dependency>
        <dependentAssembly>
            <assemblyIdentity
                    type="win32"
                    name="Microsoft.Windows.Common-Controls"
                    version="6.0.0.0"
                    processorArchitecture="*"
                    publicKeyToken="6595b64144ccf1df"
                    language="*"
            />
        </dependentAssembly>
    </dependency>
    <asmv3:application>
        <asmv3:windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">True/PM</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </asmv3:windowsSettings>
    </asmv3:application>
</assembly>
"#;

const RESOURCE: &str = r#"#include "windows.h"

1 RT_MANIFEST "{{executable_name}}.manifest"
{{bundle_icon}}

VS_VERSION_INFO VERSIONINFO
FILEVERSION     {{bundle_major_version}},{{bundle_minor_version}},{{bundle_patch_version}},0
PRODUCTVERSION  {{bundle_major_version}},{{bundle_minor_version}},{{bundle_patch_version}},0
FILEFLAGSMASK   VS_FFI_FILEFLAGSMASK
FILEFLAGS       VS_FF_DEBUG
FILEOS          VOS__WINDOWS32
FILETYPE        VFT_APP
FILESUBTYPE     VFT2_UNKNOWN
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904E4"    // Lang=US English, CharSet=Windows Multilin
        BEGIN
            VALUE "CompanyName", "{{bundle_author}}\0"
            VALUE "FileDescription", "{{bundle_name}}\0"
            VALUE "FileVersion", "{{bundle_major_version}}.{{bundle_minor_version}}.{{bundle_patch_version}}\0"
            VALUE "ProductName", "{{bundle_name}}\0"
            VALUE "ProductVersion", "{{bundle_major_version}}.{{bundle_minor_version}}.{{bundle_patch_version}}\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1252
    END
END
"#;
