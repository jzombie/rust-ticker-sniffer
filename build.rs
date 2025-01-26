#[path = "shared/lib.rs"]
mod shared;
use shared::constants::{COMPANY_SYMBOL_CSV_FILE_PATH, COMPRESSED_COMPANY_SYMBOL_FILE_NAME};

use build_resource_byte_arrays::write_byte_arrays;
// use bytes::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to fully recompile if this asset changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    // Determine the output directory
    let out_dir = env::var("OUT_DIR")?;
    let output_rust_path = PathBuf::from("src").join("dynamic_resources.rs");
    let output_bin_dir = PathBuf::from(&out_dir).join("bin");

    // Ensure the binary output directory exists
    std::fs::create_dir_all(&output_bin_dir)?;

    // Path for the binary file
    let binary_file_path = output_bin_dir.join(COMPRESSED_COMPANY_SYMBOL_FILE_NAME);

    // Open the input CSV file
    let mut input_file =
        File::open(&*COMPANY_SYMBOL_CSV_FILE_PATH).expect("Could not open the input CSV file");

    // Create a buffer to hold the compressed data
    let mut compressed_data = Vec::new();

    // Create a GzEncoder to compress the file into the buffer
    {
        let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());

        // Copy data from the input file to the encoder
        io::copy(&mut input_file, &mut encoder).expect("Failed to compress the CSV file");

        // Finish the compression process
        encoder.finish().expect("Failed to finalize compression");
    }

    // Write the compressed data to the binary file
    {
        let mut bin_file = File::create(&binary_file_path)?;
        bin_file.write_all(&compressed_data)?;
    }

    // Write the Rust code that points to the binary file
    write_byte_arrays(
        &output_rust_path,
        vec![(
            "COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY",
            compressed_data.into(),
        )],
        Some("build_resource_byte_arrays".to_string()),
    )?;

    println!(
        "Successfully compressed '{:?}', wrote binary to '{:?}', and generated Rust code at '{:?}'",
        &*COMPANY_SYMBOL_CSV_FILE_PATH, binary_file_path, output_rust_path
    );

    Ok(())
}
