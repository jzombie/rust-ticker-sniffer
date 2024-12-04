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
        // Load symbols from a test CSV file
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        // Read the content of the text file
        let text = fs::read_to_string(test_file_path).expect("Failed to read test file");

        // Extract tickers from the text
        let results = extract_tickers_from_text(&text, &symbols_map);

        // Get the expected tickers and failure reason
        let expected_tickers = get_expected_tickers(&Path::new(test_file_path));
        let expected_failure = get_expected_failure(&Path::new(test_file_path));

        // Log the file being processed
        eprintln!("Testing file: {}", test_file_path);

        if let Some(expected_failure_message) = expected_failure {
            eprintln!("Testing expected failure: {}", expected_failure_message);

            // Determine actual failure reason dynamically
            let unexpected_tickers: Vec<String> = results
                .iter()
                .filter(|ticker| !expected_tickers.contains(ticker))
                .cloned()
                .collect();

            let missing_tickers: Vec<String> = expected_tickers
                .iter()
                .filter(|ticker| !results.contains(ticker))
                .cloned()
                .collect();

            let actual_failure_reason = if !unexpected_tickers.is_empty() {
                format!("Unexpected tickers found: {:?}.", unexpected_tickers)
            } else if !missing_tickers.is_empty() {
                format!("Missing expected tickers: {:?}.", missing_tickers)
            } else {
                "No discrepancies found, but a failure was expected.".to_string()
            };

            // Validate that the actual failure reason matches the expected failure message
            assert_eq!(
                expected_failure_message, actual_failure_reason,
                "{} - Failure reason mismatch. Expected: '{}', but got: '{}'.",
                test_file_path, expected_failure_message, actual_failure_reason
            );

            // Skip further checks since failure was validated
            return;
        }

        // Regular success case validation
        assert_eq!(
            results.len(),
            expected_tickers.len(),
            "{} - Expected: {:?}, but got: {:?}",
            test_file_path,
            expected_tickers,
            results
        );

        for ticker in &expected_tickers {
            assert!(
                results.contains(ticker),
                "{} - Expected ticker {:?} was not found in results.",
                test_file_path,
                ticker
            );
        }

        for ticker in &results {
            assert!(
                expected_tickers.contains(ticker),
                "{} - Unexpected ticker {:?} found in results.",
                test_file_path,
                ticker
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
