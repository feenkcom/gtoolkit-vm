use std::fs;
use std::path::PathBuf;

pub fn try_find_image_file_in_directory(path: PathBuf) -> Option<PathBuf> {
    let files = fs::read_dir(&path).unwrap();
    let image_files: Vec<PathBuf> = files
        .filter_map(Result::ok)
        .filter(|d| {
            if let Some(e) = d.path().extension() {
                e == "image"
            } else {
                false
            }
        })
        .map(|d| d.path().to_path_buf())
        .collect();

    match image_files.len() {
        1 => Some(image_files[0].clone()),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn search_image_file_within_directories(directories: Vec<PathBuf>) -> Option<PathBuf> {
    for directory in directories {
        if let Some(image) = try_find_image_file_in_directory(directory) {
            return Some(image);
        }
    }
    None
}

#[allow(dead_code)]
#[cfg(all(
    feature = "image_picker",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
))]
pub fn pick_image_with_dialog(default_path: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = nfd2::dialog();
    let mut dialog_ref = &mut dialog;
    if let Some(ref default_path) = default_path {
        dialog_ref = dialog_ref.default_path(default_path);
    }
    dialog_ref = dialog_ref.filter("image");

    let result = dialog_ref.open().unwrap_or_else(|e| {
        panic!("{}", e);
    });

    match result {
        nfd2::Response::Okay(file_name) => {
            let file_path = PathBuf::new().join(file_name);
            if file_path.exists() {
                Some(file_path)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(not(all(
    feature = "image_picker",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
)))]
pub fn pick_image_with_dialog(_default_path: Option<PathBuf>) -> Option<PathBuf> {
    None
}

#[allow(dead_code)]
pub fn validate_user_image_file(image_name: Option<&str>) -> Option<PathBuf> {
    if let Some(image_file_name) = image_name {
        let image_path = PathBuf::from(image_file_name);
        if image_path.exists() {
            return Some(image_path);
        }
    }
    None
}
