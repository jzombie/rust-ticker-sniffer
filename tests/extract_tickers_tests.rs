mod test_utils;

use test_utils::load_symbols_from_file;
use ticker_sniffer::extract_tickers_from_text;

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
