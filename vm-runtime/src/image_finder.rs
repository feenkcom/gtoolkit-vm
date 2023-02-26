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
pub fn validate_user_image_file(image_name: Option<&str>) -> Option<PathBuf> {
    if let Some(image_file_name) = image_name {
        let image_path = PathBuf::from(image_file_name);
        if image_path.exists() {
            return Some(image_path);
        }
    }
    None
}
