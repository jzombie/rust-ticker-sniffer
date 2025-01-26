#[path = "shared/lib.rs"]
mod shared;
use shared::constants::COMPANY_SYMBOL_CSV_FILE_PATH;

use embed_bytes::write_byte_arrays;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{self};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure that Cargo re-runs the build script if the input file changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    // Set the output directory for generated files
    let output_rust_path = PathBuf::from("embed");

    // Open the input CSV file
    let mut input_file =
        File::open(&*COMPANY_SYMBOL_CSV_FILE_PATH).expect("Could not open the input CSV file");

    // Create a buffer to hold the compressed data
    let mut compressed_data = Vec::new();

    // Compress the data with GzEncoder
    {
        let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());
        io::copy(&mut input_file, &mut encoder).expect("Failed to compress the CSV file");
        encoder.finish().expect("Failed to finalize compression");
    }

    // Write the compressed data to the `embed` directory
    write_byte_arrays(
        &output_rust_path,
        vec![(
            "COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY",
            compressed_data.into(),
        )],
    )?;

    println!("Files successfully written to {:?}", output_rust_path);

    Ok(())
}
