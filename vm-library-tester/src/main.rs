extern crate clap;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Options {
    #[clap(parse(from_os_str))]
    library: PathBuf,
    #[clap(long, value_parser)]
    symbol: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options: Options = Options::parse();

    let lib = unsafe { libloading::Library::new(options.library)? };
    if let Some(ref symbol) = options.symbol {
        unsafe { lib.get::<libloading::Symbol<unsafe extern "C" fn()>>(symbol.as_str().as_ref())? };
    }
    Ok(())
}
