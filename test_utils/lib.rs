use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::{fs, path::Path};
use ticker_sniffer::extract_tickers_from_text;

/// Utility to load symbols from a CSV file for testing and benchmarking.
pub fn load_symbols_from_file(
    file_path: &str,
) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
    let mut symbols_map = HashMap::new();
    let mut reader = Reader::from_path(file_path)?;

    for record in reader.records() {
        let record = record?;
        if record.len() == 2 {
            let symbol = record.get(0).unwrap().to_uppercase();
            let company_name = record.get(1).map(|name| name.to_string());
            symbols_map.insert(symbol, company_name);
        } else {
            eprintln!("Skipping invalid row: {:?}", record);
        }
    }

    Ok(symbols_map)
}

// Helper function to get the expected tickers from the text file
pub fn get_expected_tickers(file_path: &Path) -> Vec<String> {
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
pub fn get_expected_failure(file_path: &Path) -> Option<String> {
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
pub fn run_test_for_file(test_file_path: &str) -> usize {
    // Load symbols from a test CSV file
    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    // Read the content of the text file
    let raw_text = fs::read_to_string(test_file_path).expect("Failed to read test file");

    // Filter out lines starting with 'EXPECTED:' or 'EXPECTED_FAILURE:'
    let filtered_text: String = raw_text
        .lines()
        .filter(|line| !line.trim_start().starts_with("EXPECTED:"))
        .filter(|line| !line.trim_start().starts_with("EXPECTED_FAILURE:"))
        .collect::<Vec<&str>>()
        .join("\n");

    // Log the filtered text
    eprintln!("Filtered text: {}", filtered_text);

    // Extract tickers from the filtered text
    let results = extract_tickers_from_text(&filtered_text, &symbols_map);

    // Get the expected tickers and failure reason
    let expected_tickers = get_expected_tickers(&Path::new(test_file_path));
    let expected_failure = get_expected_failure(&Path::new(test_file_path));

    // Log the file being processed
    eprintln!("Testing file: {}", test_file_path);

    // Check for duplicate tickers in the results
    let mut ticker_counts = std::collections::HashMap::new();
    for ticker in &results {
        *ticker_counts.entry(ticker).or_insert(0) += 1;
    }
    let duplicate_tickers: Vec<&String> = ticker_counts
        .iter()
        .filter(|(_, &count)| count > 1)
        .map(|(ticker, _)| *ticker)
        .collect();

    let mut error_count = 0;

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
        } else if !duplicate_tickers.is_empty() {
            format!("Duplicate tickers found: {:?}.", duplicate_tickers)
        } else {
            "No discrepancies found, but a failure was expected.".to_string()
        };

        if expected_failure_message != actual_failure_reason {
            error_count += 1; // Increment error count for failure reason mismatch
        }

        // Validate that the actual failure reason matches the expected failure message
        assert_eq!(
            expected_failure_message, actual_failure_reason,
            "{} - Failure reason mismatch. Expected: '{}', but got: '{}'.",
            test_file_path, expected_failure_message, actual_failure_reason
        );

        // Skip further checks since failure was validated
        return error_count; // Return early with errors
    }

    // Regular success case validation
    if results.len() != expected_tickers.len() {
        error_count += 1; // Increment error count for length mismatch
    }

    for ticker in &expected_tickers {
        if !results.contains(ticker) {
            error_count += 1; // Increment error count for missing expected tickers
        }
    }

    if !duplicate_tickers.is_empty() {
        error_count += duplicate_tickers.len(); // Increment for duplicate tickers
    }

    for ticker in &results {
        if !expected_tickers.contains(ticker) {
            error_count += 1; // Increment error count for unexpected tickers
        }
    }

    // Keep all existing assertions intact
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

    assert!(
        duplicate_tickers.is_empty(),
        "{} - Duplicate tickers found in results: {:?}",
        test_file_path,
        duplicate_tickers
    );

    for ticker in &results {
        assert!(
            expected_tickers.contains(ticker),
            "{} - Unexpected ticker {:?} found in results.",
            test_file_path,
            ticker
        );
    }

    error_count // Return the total number of errors
}
