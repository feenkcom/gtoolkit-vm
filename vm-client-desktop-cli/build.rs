extern crate embed_resource;

fn main() {
    match std::env::var("VM_CLIENT_EMBED_RESOURCES") {
        Ok(resources) => {
            for resource in resources.split(",") {
                embed_resource::compile(resource);
                println!("cargo:rerun-if-changed={}", resource);
            }
        }
        Err(_) => {}
    }
}
