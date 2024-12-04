mod test_utils;

use std::{fs, path::Path};
use test_utils::load_symbols_from_file;
use ticker_sniffer::{extract_tickers_from_text, tokenize};

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_extract_tickers_with_file() {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        // Example test file path (adjust if necessary)
        let test_file_path = "tests/test_files/test_file_1.txt";

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
            "Mismatch in the number of extracted tickers"
        );

        // Ensure that all expected tickers are found in the extracted results
        for ticker in &expected_tickers {
            assert!(
                results.contains(ticker),
                "Missing expected ticker: {}",
                ticker
            );
        }

        // Ensure that there are no unexpected tickers in the results
        for ticker in &results {
            assert!(
                expected_tickers.contains(ticker),
                "Unexpected ticker found: {}",
                ticker
            );
        }
    }

    #[test]
    fn test_extract_tickers_with_alternatives() {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        // Example test file path (adjust if necessary)
        let test_file_path = "tests/test_files/test_file_2.txt";

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
            "Mismatch in the number of extracted tickers"
        );

        // Ensure that all expected tickers are found in the extracted results
        for ticker in &expected_tickers {
            assert!(
                results.contains(ticker),
                "Missing expected ticker: {}",
                ticker
            );
        }

        // Ensure that there are no unexpected tickers in the results
        for ticker in &results {
            assert!(
                expected_tickers.contains(ticker),
                "Unexpected ticker found: {}",
                ticker
            );
        }
    }
}
