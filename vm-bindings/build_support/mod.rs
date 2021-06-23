mod builder;
pub use builder::Builder;

#[cfg(target_os = "macos")]
pub mod mac;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub use mac::MacBuilder as PlatformBuilder;

#[cfg(target_os = "windows")]
pub use windows::WindowsBuilder as PlatformBuilder;

#[cfg(all(not(target_os = "macos"), not(target_os = "windows"),))]
compile_error!("The platform you're compiling for is not supported");
