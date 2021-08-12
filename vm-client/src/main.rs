#![windows_subsystem = "windows"]

extern crate dirs;
extern crate nfd2;
extern crate thiserror;
extern crate vm_bindings;
#[macro_use]
extern crate log;

mod error;
mod image_finder;

mod application;
mod application_options;
mod platform;
mod working_directory;

use clap::Clap;

use user_error::{UserFacingError, UFE};

pub use crate::application::Application;
pub use crate::application_options::AppOptions;
pub use crate::error::*;

fn run() -> Result<()> {
    // we should read options and canonicalize the image path before changing current directory
    let mut options: AppOptions = AppOptions::parse();
    options.canonicalize()?;

    #[cfg(target_os = "macos")]
    if let Err(error) = platform::mac::translocation::un_translocate() {
        error!("Failed to un-translocate the app due to {}", error);
    }

    let application = Application::new(options)?;
    application.start()?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        let error: Box<dyn std::error::Error> = Box::new(error);
        let user_facing_error: UserFacingError = error.into();
        user_facing_error.print_and_exit();
    }
}
