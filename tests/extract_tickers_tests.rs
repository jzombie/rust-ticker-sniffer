use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use ticker_sniffer::extract_tickers_from_text;

/// Loads symbols from a file for testing purposes.
fn load_symbols_from_file(file_path: &str) -> HashMap<String, Option<String>> {
    let mut symbols_map = HashMap::new();
    let file = File::open(file_path).expect("Failed to open test symbols file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() == 2 {
            symbols_map.insert(parts[0].to_uppercase(), Some(parts[1].to_string()));
        } else {
            symbols_map.insert(parts[0].to_uppercase(), None);
        }
    }

    symbols_map
}

#[test]
fn test_extract_tickers_with_file() {
    // Load symbols from a test CSV file
    let symbols_map = load_symbols_from_file("tests/test_symbols.csv");

    let text = "AAPL is performing well, but MSFT is also a strong contender.";

    let results = extract_tickers_from_text(text, &symbols_map);

    // Check that the correct tickers are matched
    assert_eq!(results.len(), 2);
    assert!(results.contains(&"AAPL".to_string()));
    assert!(results.contains(&"MSFT".to_string()));
}
