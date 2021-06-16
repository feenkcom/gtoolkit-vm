mod builder;
pub use builder::Builder;

#[cfg(target_os = "macos")]
pub mod mac;
#[cfg(target_os = "macos")]
pub use mac::MacBuilder as PlatformBuilder;

#[cfg(all(not(target_os = "macos"),))]
compile_error!("The platform you're compiling for is not supported");
