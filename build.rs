use flate2::write::GzEncoder;
use flate2::Compression;
use shared::constants::{
    CODE_AUTOGEN_PREFIX, COMPANY_SYMBOL_CSV_FILE_PATH, COMPRESSED_COMPANY_SYMBOL_FILE_NAME,
};
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

/// This is a standalone utility binary used for preprocessing the company
/// symbol list into a compressed format before building the main project.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to fully recompile if this asset changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    // Get the output directory from the Cargo environment
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let out_dir_path = PathBuf::from(out_dir);

    // Construct the output file path in the `OUT_DIR`
    let output_file_path = out_dir_path.join(format!(
        "{}{}",
        CODE_AUTOGEN_PREFIX, COMPRESSED_COMPANY_SYMBOL_FILE_NAME
    ));

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&out_dir_path).expect("Failed to create output directory");

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

    println!(
        "Successfully compressed '{:?}' to '{}'",
        &*COMPANY_SYMBOL_CSV_FILE_PATH,
        output_file_path.display()
    );

    Ok(())
}
