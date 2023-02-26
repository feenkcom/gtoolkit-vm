use crate::{ApplicationError, Result};
use std::path::PathBuf;

#[cfg(target_os = "macos")]
pub fn executable_working_directory() -> Result<PathBuf> {
    // working_directory/Application.app/Contents/MacOS/executable
    let mut app_dir = std::env::current_exe()?;

    // working_directory/Application.app/Contents/MacOS/
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    // working_directory/Application.app/Contents/
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    // working_directory/Application.app/
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    // working_directory
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    Ok(app_dir)
}

#[cfg(all(not(target_os = "macos"),))]
pub fn executable_working_directory() -> Result<PathBuf> {
    // working_directory/bin/executable
    let mut app_dir = std::env::current_exe()?;

    // working_directory/bin/
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    // working_directory/
    app_dir = app_dir
        .parent()
        .ok_or_else(|| ApplicationError::NoParentDirectory(app_dir.clone()))?
        .to_path_buf();

    Ok(app_dir)
}
