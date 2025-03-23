use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this script if the assets change
    println!("cargo:rerun-if-changed=assets/");

    // Get the output directory from Cargo
    let out_dir = env::var("OUT_DIR").unwrap();

    // Copy SVG file to output directory if needed for runtime use
    let dest_path = Path::new(&out_dir).join("mimic-rs-logo.svg");
    fs::copy("assets/mimic-rs-logo.svg", dest_path).expect("Failed to copy SVG file");

    println!("cargo:warning=SVG logo copied to output directory");
}
