#[path = "../test_utils/lib.rs"]
mod test_utils;

use std::collections::HashMap;
use std::fs::read_dir;
use test_utils::run_test_for_file;
use ticker_sniffer::{ResultBiasAdjuster, DEFAULT_WEIGHTS};

// Declare a module-level variable for ResultBiasAdjuster
static mut CONTEXT_ATTENTION: Option<ResultBiasAdjuster> = None;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_from_multiple_files() {
        // SAFETY: We need to ensure no data races while accessing CONTEXT_ATTENTION
        unsafe {
            if CONTEXT_ATTENTION.is_none() {
                // TODO: Initialize with some weights
                // CONTEXT_ATTENTION = Some(ResultBiasAdjuster::new(256));
                CONTEXT_ATTENTION = Some(ResultBiasAdjuster::from_weights(HashMap::from([
                    (15726725667165516449, 0.0),
                    (18175059305241253125, 0.0),
                    (6760917874204056577, 0.0),
                    (2949345933018568219, 0.0),
                    (2963807753026003782, 0.0),
                    (13343208097864498901, 0.0),
                    (10774098009159108967, 0.0),
                    (18009285742348921912, 0.0),
                ])));
            }

            let result_bias_adjuster = CONTEXT_ATTENTION.as_ref().unwrap();

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
                        result_bias_adjuster,
                    );
                }
            }
        }
    }
}
