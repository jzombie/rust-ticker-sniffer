use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs::File;
use std::io;
use ticker_sniffer_common_lib::constants::CODE_AUTOGEN_PREFIX;
use ticker_sniffer_common_lib::types::CompanySymbolList;

// TODO: Refactor
const COMPANY_SYMBOL_FILE_PATH: &str = "../data/company_symbol_list.csv";

pub fn run_build_utils() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to fully recompile if this asset changes
    println!("cargo:rerun-if-changed={}", COMPANY_SYMBOL_FILE_PATH);

    let output_file_path = format!("{}company_symbol_list.csv.gz", CODE_AUTOGEN_PREFIX);

    // Open the input CSV file
    let input_file =
        File::open(COMPANY_SYMBOL_FILE_PATH).expect("Could not open the input CSV file");

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
        "Successfully compressed '{}' to '{}'",
        COMPANY_SYMBOL_FILE_PATH, output_file_path
    );

    Ok(())
}
