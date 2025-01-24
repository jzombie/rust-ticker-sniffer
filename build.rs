use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::{self, File};
use std::io;

#[path = "shared/lib.rs"]
mod shared;
use shared::constants::{
    CODE_AUTOGEN_PREFIX, COMPANY_SYMBOL_CSV_FILE_PATH, COMPRESSED_COMPANY_SYMBOL_FILE_NAME,
};

/// This is a standalone utility binary used for preprocessing the company
/// symbol list into a compressed format before building the main project.
fn main() -> io::Result<()> {
    // Set a conditional compilation flag for builds
    println!("cargo:rustc-cfg=build_env");

    // Tell Cargo to fully recompile if this asset changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    // Get OUT_DIR
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");

    // Ensure the directory exists
    fs::create_dir_all(&out_dir).expect("Failed to create OUT_DIR");

    // Build the output file path using OUT_DIR
    let output_file_path = format!(
        "{}/{}{}",
        out_dir, CODE_AUTOGEN_PREFIX, COMPRESSED_COMPANY_SYMBOL_FILE_NAME
    );

    // Open the input CSV file
    let input_file =
        File::open(&*COMPANY_SYMBOL_CSV_FILE_PATH).expect("Could not open the input CSV file");

    // Open the output file for writing
    let output_file =
        File::create(&output_file_path).expect("Could not create the output compressed file");

    // Create a GzEncoder to compress the file
    let mut encoder = GzEncoder::new(output_file, Compression::default());

    // Copy data from the input file to the encoder
    io::copy(&mut &input_file, &mut encoder).expect("Failed to compress the CSV file");

    // Finish the compression process
    encoder.finish().expect("Failed to finalize compression");

    println!("File compressed to: {}", output_file_path);
    Ok(())
}
