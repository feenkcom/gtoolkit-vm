use nfd2::{dialog, Response};
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

pub fn search_image_file_nearby() -> Option<PathBuf> {
    let image_file = std::env::current_exe().map_or(None, |path| {
        path.parent().map_or(None, |exe_path| {
            try_find_image_file_in_directory(exe_path.to_path_buf())
        })
    });

    if image_file.is_some() {
        return image_file;
    }

    std::env::current_dir().map_or(None, |path| try_find_image_file_in_directory(path))
}

pub fn pick_image_with_dialog() -> Option<PathBuf> {
    let result = dialog().filter("image").open().unwrap_or_else(|e| {
        panic!("{}", e);
    });

    match result {
        Response::Okay(file_name) => {
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

pub fn validate_user_image_file(image_name: Option<&str>) -> Option<PathBuf> {
    if let Some(image_file_name) = image_name {
        let image_path = PathBuf::new().join(image_file_name);
        if image_path.exists() {
            return Some(image_path);
        }
    }
    None
}
