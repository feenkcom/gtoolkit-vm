#![windows_subsystem = "windows"]

#[macro_use]
extern crate vm_bindings;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_traits;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate default_env;

pub(crate) mod platform;
mod runtime;

pub use runtime::*;

use clap::Parser;

use user_error::{UserFacingError, UFE};

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

    let application = Application::new(options)?;
    application.start()?;

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(error) = run() {
        let error: Box<dyn std::error::Error> = Box::new(error);
        let user_facing_error: UserFacingError = error.into();
        user_facing_error.print_and_exit();
    }
}
