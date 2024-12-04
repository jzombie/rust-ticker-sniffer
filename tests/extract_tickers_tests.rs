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

    // Helper function to run the test for each file in the directory
    fn run_test_for_file(test_file_path: &str) {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        // Read the content of the text file
        let text = fs::read_to_string(test_file_path).expect("Failed to read test file");

        // Extract tickers from the text
        let results = extract_tickers_from_text(&text, &symbols_map);

        // Print the results to check what's being extracted
        println!("{:?}", results);

        // Get the expected tickers from the same file
        let expected_tickers = get_expected_tickers(&Path::new(test_file_path));

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

        // Read all files in the test directory
        let entries = read_dir(test_dir).expect("Failed to read test directory");

        // Iterate over each file and run the test
        for entry in entries {
            let entry = entry.expect("Failed to read entry in test directory");
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "txt").unwrap_or(false) {
                run_test_for_file(path.to_str().unwrap());
            }
        }
    }
}
