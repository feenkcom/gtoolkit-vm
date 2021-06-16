mod builder;
pub use builder::Builder;

#[cfg(target_os = "macos")]
pub mod mac;
#[cfg(target_os = "macos")]
pub use mac::MacBuilder as PlatformBuilder;

pub fn create_builder() -> Box<dyn Builder> {
    Box::new(PlatformBuilder::default())
}

#[cfg(all(not(target_os = "macos"),))]
compile_error!("The platform you're compiling for is not supported");
