#[cfg(target_os = "macos")]
pub fn ensure_working_directory() {
    let app_dir = std::env::current_exe().map_or(None, |exe_path| {
        exe_path.parent().map_or(None, |parent| {
            parent.parent().map_or(None, |parent| {
                parent.parent().map_or(None, |parent| {
                    parent
                        .parent()
                        .map_or(None, |parent| Some(parent.to_path_buf()))
                })
            })
        })
    });
    if app_dir.is_some() {
        std::env::set_current_dir(app_dir.unwrap()).unwrap();
    }
}

#[cfg(all(not(target_os = "macos"),))]
pub fn ensure_working_directory() {
    let app_dir = std::env::current_exe().map_or(None, |exe_path| {
        exe_path.parent().map_or(None, |parent| {
            parent
                .parent()
                .map_or(None, |parent| Some(parent.to_path_buf()))
        })
    });
    if app_dir.is_some() {
        std::env::set_current_dir(app_dir.unwrap()).unwrap();
    }
}
