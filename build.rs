#[path = "dev_shared/lib.rs"]
mod dev_shared;
use dev_shared::constants::COMPANY_SYMBOL_CSV_FILE_PATH;

use embed_resources::{Resource, ResourceContainer};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure that Cargo re-runs the build script if the input file changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    // Set the output directory for generated files
    let output_rust_path = PathBuf::from("embed");

    let mut resource_container = ResourceContainer::new(&output_rust_path);

    resource_container.add_resource(
        "COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY",
        Resource::File(
            COMPANY_SYMBOL_CSV_FILE_PATH
                .to_str()
                .expect("Path contains invalid UTF-8 characters")
                .to_string(),
        ),
        true,
    );

    resource_container.embed_all()?;

    println!("Files successfully written to {:?}", output_rust_path);

    Ok(())
}
