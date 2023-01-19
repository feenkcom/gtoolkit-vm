use std::rc::Rc;

use platforms::platform::Platform;

pub use builder::{ArchBits, Builder, FamilyOS, HostOS, TargetOS};
pub(crate) use windows::WindowsBuilder;

mod builder;
mod linux;
mod mac;
mod other;
mod windows;

pub fn for_target_triplet(triplet: &str) -> anyhow::Result<Rc<dyn Builder>> {
    if let Some(platform) = Platform::find(triplet) {
        let os = TargetOS::from(platform);
        let builder = match FamilyOS::from(os) {
            FamilyOS::Unix => linux::LinuxBuilder::new(platform.clone()).boxed(),
            FamilyOS::Apple => mac::MacBuilder::new(platform.clone()).boxed(),
            FamilyOS::Windows => windows::WindowsBuilder::new(platform.clone()).boxed(),
            FamilyOS::Other => other::OtherBuilder::new(platform.clone()).boxed(),
        };
        Ok(builder)
    } else {
        Err(anyhow!(
            "The target triplet ({}) you're compiling for is not supported",
            triplet
        ))
    }
}
