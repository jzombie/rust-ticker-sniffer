use crate::types::TickerSymbol;
use std::collections::HashMap;

/// Counts the frequency of ticker symbols in the given list.
///
/// # Arguments
/// * `ticker_symbols` - A slice of ticker symbols to analyze.
///
/// # Returns
/// * A `HashMap` where the keys are ticker symbols and the values are their
///   respective frequencies.
///
/// # Example
/// ```
/// let symbols = vec!["AAPL".to_string(), "GOOGL".to_string(), "AAPL".to_string()];
/// let frequencies = count_ticker_symbol_frequencies(&symbols);
/// assert_eq!(frequencies.get("AAPL"), Some(&2));
/// assert_eq!(frequencies.get("GOOGL"), Some(&1));
/// ```
pub fn count_ticker_symbol_frequencies(
    ticker_symbols: &[TickerSymbol],
) -> HashMap<TickerSymbol, usize> {
    let mut frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

    for ticker_symbol in ticker_symbols {
        *frequencies.entry(ticker_symbol.clone()).or_insert(0) += 1;
    }

    frequencies
}
