mod test_utils;

use std::{fs, path::Path};
use test_utils::load_symbols_from_file;
use ticker_sniffer::extract_tickers_from_text;

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::read_dir;
    // Helper function to get the expected tickers from the text file
    fn get_expected_tickers(file_path: &Path) -> Vec<String> {
        // Read the content of the text file
        let content = fs::read_to_string(file_path).expect("Failed to read test file");

        // Extract tickers from lines starting with EXPECTED:
        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with("EXPECTED:") {
                    Some(line.replace("EXPECTED:", "").trim().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    // Helper function to check if the file has an EXPECTED_FAILURE line
    fn get_expected_failure(file_path: &Path) -> Option<String> {
        let content = fs::read_to_string(file_path).expect("Failed to read test file");

        content.lines().find_map(|line| {
            let line = line.trim();
            if line.starts_with("EXPECTED_FAILURE:") {
                Some(line.replace("EXPECTED_FAILURE:", "").trim().to_string())
            } else {
                None
            }
        })
    }

    // Helper function to run the test for each file in the directory
    fn run_test_for_file(test_file_path: &str) {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        // Read the content of the text file
        let text = fs::read_to_string(test_file_path).expect("Failed to read test file");

        // Extract tickers from the text
        let results = extract_tickers_from_text(&text, &symbols_map);

        // Get the expected tickers from the same file
        let expected_tickers = get_expected_tickers(&Path::new(test_file_path));
        let expected_failure = get_expected_failure(&Path::new(test_file_path));

        // If an expected failure is marked, then the test should fail deliberately
        if let Some(reason) = expected_failure {
            // We want to assert that 'BRK.B' does not match 'BRK-B' in the failure cases.
            println!("Testing expected failure: {}", reason);
            assert!(
                !results.contains(&"BRK-B".to_string()) || !results.contains(&"BRK.B".to_string()),
                "Expected failure: {}, but tickers matched: {:?}",
                reason,
                results
            );
            return; // Skip further checks since we expect a failure
        }

        // Ensure that the correct number of tickers are matched
        assert_eq!(
            results.len(),
            expected_tickers.len(),
            "Mismatch in the number of extracted tickers in file: {}",
            test_file_path
        );

        // Ensure that all expected tickers are found in the extracted results
        for ticker in &expected_tickers {
            assert!(
                results.contains(ticker),
                "Missing expected ticker: {} in file: {}",
                ticker,
                test_file_path
            );
        }

        // Ensure that there are no unexpected tickers in the results
        for ticker in &results {
            assert!(
                expected_tickers.contains(ticker),
                "Unexpected ticker found: {} in file: {}",
                ticker,
                test_file_path
            );
        }
    }

    #[test]
    fn test_extract_tickers_from_multiple_files() {
        // Directory containing the test files
        let test_dir = "tests/test_files";

        // Read all files in the directory
        let files = read_dir(test_dir).expect("Failed to read test files directory");

        for file in files {
            let file = file.expect("Failed to read file");
            let file_path = file.path();

            // Run the test for each file (if it is a file)
            if file_path.is_file() {
                run_test_for_file(file_path.to_str().unwrap());
            }
        }
    }
}
