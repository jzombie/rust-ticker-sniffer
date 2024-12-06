#[path = "../test_utils/lib.rs"]
mod test_utils;

use std::fs::read_dir;
use test_utils::run_test_for_file;
use ticker_sniffer::Weights;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_from_multiple_files() {
        const WEIGHTS: Weights = Weights {
            continuity: 0.025,
            mismatched_letter_penalty: 1.0,
            mismatched_word_penalty: 0.3,
            match_score_threshold: 0.25,
            symbol_abbr_threshold: 0.8,
            // continuity: 0.3688305957567424,
            // coverage_input: 0.026040188967873246,
            // coverage_company: 0.5971237581795172,
            // match_score_threshold: 1.6376519441299855,
        };

        // Directory containing the test files
        let test_dir = "tests/test_files";

        // Read all files in the directory
        let files = read_dir(test_dir).expect("Failed to read test files directory");

        for file in files {
            let file = file.expect("Failed to read file");
            let file_path = file.path();

            // Run the test for each file (if it is a file)
            if file_path.is_file() {
                run_test_for_file(file_path.to_str().unwrap(), true, WEIGHTS);
            }
        }
    }
}
