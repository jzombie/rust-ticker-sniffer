#[path = "dev_shared/lib.rs"]
mod dev_shared;
use dev_shared::constants::COMPANY_SYMBOL_CSV_FILE_PATH;

use embed_resources::{Resource, ResourceContainer};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure that Cargo re-runs the build script if the input file changes
    println!(
        "cargo:rerun-if-changed={:?}",
        &*COMPANY_SYMBOL_CSV_FILE_PATH
    );

    let struct_output_path = Path::new("src/structs/resource_container.rs"); // Struct file path
    let struct_name = "ResourceContainer";

    let mut resource_container = ResourceContainer::new(struct_output_path, struct_name);

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

    Ok(())
}
