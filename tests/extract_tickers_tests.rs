mod test_utils;

use test_utils::load_symbols_from_file;
use ticker_sniffer::extract_tickers_from_text;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_with_file() {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        let text = "AAPL is performing well, but MSFT is also a strong contender.";

        let results = extract_tickers_from_text(text, &symbols_map);

        // Print the results to check what's being extracted
        println!("{:?}", results);

        // Check that the correct tickers are matched
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"AAPL".to_string()));
        assert!(results.contains(&"MSFT".to_string()));
    }

    #[test]
    fn test_extract_tickers_with_alternatives() {
        // Load symbols from a test CSV file and handle the Result
        let symbols_map = load_symbols_from_file("tests/test_symbols.csv")
            .expect("Failed to load symbols from CSV");

        let text = "BRK.B is a great stock, and so is MSFT.";

        let results = extract_tickers_from_text(text, &symbols_map);

        // Print the results to check what's being extracted
        println!("{:?}", results);

        // Check that "BRK.B" is matched to "BRK-B"
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"BRK-B".to_string()));
        assert!(results.contains(&"MSFT".to_string()));
    }
}
