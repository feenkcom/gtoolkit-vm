#![windows_subsystem = "windows"]

#[macro_use]
extern crate log;

use clap::Parser;

use crate::application::Application;
use crate::application_options::AppOptions;
use user_error::{UserFacingError, UFE};
use vm_runtime::{print_version, ApplicationError, Result};

mod application;
mod application_options;
mod platform;

fn run() -> Result<()> {
    // we should read options and canonicalize the image path before changing current directory
    let mut options: AppOptions = AppOptions::parse();
    if options.version {
        print_version();
        return Ok(());
    }

    options.canonicalize()?;

    #[cfg(target_os = "macos")]
    if let Err(error) = platform::mac::translocation::un_translocate() {
        error!("Failed to un-translocate the app due to {}", error);
    }

    #[cfg(target_os = "linux")]
    {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let application = Application::new(options)?;
    application.start()?;

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(error) = run() {
        handle_application_error(error)
    }
}

#[allow(dead_code)]
#[cfg(all(
    feature = "error-dialog",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
))]
pub fn handle_application_error(error: ApplicationError) {
    use native_dialog::{MessageDialog, MessageType};

    let error: Box<dyn std::error::Error> = Box::new(error);
    let user_facing_error: UserFacingError = error.into();

    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(user_facing_error.summary().as_str())
        .set_text(user_facing_error.to_string().as_str())
        .show_alert()
        .unwrap();

    std::process::exit(1)
}

#[cfg(not(all(
    feature = "error-dialog",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
)))]
pub fn handle_application_error(error: ApplicationError) {
    let error: Box<dyn std::error::Error> = Box::new(error);
    let user_facing_error: UserFacingError = error.into();
    user_facing_error.print_and_exit();
}
