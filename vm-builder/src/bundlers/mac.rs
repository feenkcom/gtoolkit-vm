use crate::bundlers::Bundler;
use crate::BuildOptions;
use std::fs;
use std::path::Path;
use std::fs::File;

pub struct MacBundler {}

impl MacBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for MacBundler {
    fn bundle(&self, configuration: &BuildOptions) {
        let bundle_location = self.bundle_location(configuration);
        let app_name = self.app_name(configuration);

        let app_dir = bundle_location.join(Path::new(&format!("{}.app", &app_name)));
        let contents_dir = app_dir.join(Path::new("Contents"));
        let resources_dir = contents_dir.join(Path::new("Resources"));
        let macos_dir = contents_dir.join(Path::new("MacOS"));

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&contents_dir).unwrap();
        fs::create_dir(&resources_dir).unwrap();
        fs::create_dir(&macos_dir).unwrap();

        let target_executable_path =
            macos_dir.join(Path::new(&self.executable_name(configuration)));

        fs::copy(
            self.compiled_executable_path(configuration),
            target_executable_path,
        )
        .unwrap();

        fs_extra::copy_items(
            &vec![self.libraries_path(configuration)],
            macos_dir,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();

        let info_plist_template = mustache::compile_str(INFO_PLIST).unwrap();
        let info = Info {
            bundle_name: self.app_name(configuration),
            bundle_display_name: self.app_name(configuration),
            executable_name: self.executable_name(configuration),
            bundle_identifier: self.bundle_identifier(configuration),
            bundle_version: self.bundle_version(configuration)
        };

        let mut file = File::create(contents_dir.join(Path::new("Info.plist"))).unwrap();
        info_plist_template.render(&mut file, &info).unwrap();
    }
}

#[derive(Serialize)]
struct Info {
    bundle_name: String,
    bundle_display_name: String,
    executable_name: String,
    bundle_identifier: String,
    bundle_version: String
}

const INFO_PLIST: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>English</string>
  <key>CFBundleDisplayName</key>
  <string>{{bundle_display_name}}</string>
  <key>CFBundleExecutable</key>
  <string>{{executable_name}}</string>
  <key>CFBundleIdentifier</key>
  <string>{{bundle_identifier}}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{{bundle_name}}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{{bundle_version}}</string>
  <key>CFBundleVersion</key>
  <string>{{bundle_version}}</string>
  <key>CSResourcesFileMapped</key>
  <true/>
  <key>LSRequiresCarbon</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSEnvironment</key>
	<dict>
	<key>WANTS_INTERACTIVE_SESSION</key>
	<string>true</string>
	</dict>
</dict>
</plist>
"#;