use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("telemetry-cache.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let classes = include!("./build_support/classes.in");
    let mut builder = phf_codegen::Map::<&[u8]>::new();
    for (index, class) in classes.iter().enumerate() {
        builder.entry(class.as_bytes(), format!("{}", index).as_str());
    }

    writeln!(
        &mut file,
        "static CLASSES_MAP: phf::Map<&'static [u8], usize> = \n{};\n",
        builder.build()
    )
    .unwrap();
    writeln!(
        &mut file,
        "static CLASSES: &[&str] = {};\n",
        include_str!("./build_support/classes.in")
    )
    .unwrap();

    let selectors = include!("./build_support/selectors.in");
    let mut builder = phf_codegen::Map::<&[u8]>::new();
    for (index, selector) in selectors.iter().enumerate() {
        builder.entry(selector.as_bytes(), format!("{}", index).as_str());
    }

    writeln!(
        &mut file,
        "static SELECTORS_MAP: phf::Map<&'static [u8], usize> = \n{};\n",
        builder.build()
    )
    .unwrap();
    writeln!(
        &mut file,
        "static SELECTORS: &[&str] = {};\n",
        include_str!("./build_support/selectors.in")
    )
    .unwrap();
}
