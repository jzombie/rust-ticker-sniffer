#[path = "../test_utils/lib.rs"]
mod test_utils;

use std::fs::read_dir;
use test_utils::run_test_for_file;
use ticker_sniffer::{ContextAttention, DEFAULT_WEIGHTS};

// Declare a module-level variable for ContextAttention
static mut CONTEXT_ATTENTION: Option<ContextAttention> = None;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_from_multiple_files() {
        // SAFETY: We need to ensure no data races while accessing CONTEXT_ATTENTION
        unsafe {
            if CONTEXT_ATTENTION.is_none() {
                CONTEXT_ATTENTION = Some(ContextAttention::new(256));
            }

            let context_attention = CONTEXT_ATTENTION.as_ref().unwrap();

            // Directory containing the test files
            let test_dir = "tests/test_files";

            // Read all files in the directory
            let files = read_dir(test_dir).expect("Failed to read test files directory");

            for file in files {
                let file = file.expect("Failed to read file");
                let file_path = file.path();

                // Run the test for each file (if it is a file)
                if file_path.is_file() {
                    run_test_for_file(
                        file_path.to_str().unwrap(),
                        true,
                        DEFAULT_WEIGHTS,
                        context_attention,
                    );
                }
            }
        }
    }
}
