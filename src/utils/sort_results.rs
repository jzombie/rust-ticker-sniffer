use crate::types::{TickerSymbol, TickerSymbolFrequency, TickerSymbolFrequencyMap};

/// Sorts a mapping of ticker symbols to their frequencies.
///
/// This function takes a `TickerSymbolFrequencyMap`, which is a mapping of
/// ticker symbols to their occurrence frequencies, and returns a sorted
/// vector of `(TickerSymbol, TickerSymbolFrequency)` pairs.
///
/// ### Sorting Order:
/// - **Primary:** Sorts by frequency in descending order (higher frequency first).
/// - **Secondary:** If two symbols have the same frequency, sorts by ticker
///   symbol in ascending lexicographical order for deterministic ordering.
///
/// ### Parameters:
/// - `results`: A `TickerSymbolFrequencyMap`, which is a `HashMap`
///   where the key is a `TickerSymbol` (e.g., a stock ticker)
///   and the value is a `TickerSymbolFrequency` (e.g., how often it appeared).
///
/// ### Returns:
/// - A `Vec` of `(TickerSymbol, TickerSymbolFrequency)` tuples,
///   sorted as described above.
///
/// ### Example:
/// ```rust
/// use std::collections::HashMap;
/// use ticker_sniffer::types::{TickerSymbol, TickerSymbolFrequency, TickerSymbolFrequencyMap};
/// use ticker_sniffer::sort_results;
///
/// let mut results: TickerSymbolFrequencyMap = HashMap::new();
/// results.insert("AAPL".to_string(), 10);
/// results.insert("TSLA".to_string(), 15);
/// results.insert("GOOGL".to_string(), 10);
///
/// let sorted = sort_results(results);
/// assert_eq!(sorted, vec![
///     ("TSLA".to_string(), 15),
///     ("AAPL".to_string(), 10),
///     ("GOOGL".to_string(), 10)
/// ]);
/// ```
pub fn sort_results(
    results: TickerSymbolFrequencyMap,
) -> Vec<(TickerSymbol, TickerSymbolFrequency)> {
    // Convert the HashMap into a Vec and sort it by frequency (descending),
    // then by ticker symbol (ascending) for deterministic order.
    let mut sorted_results: Vec<(TickerSymbol, TickerSymbolFrequency)> = results
        .iter()
        .map(|(ticker_symbol, frequency)| (ticker_symbol.to_owned(), frequency.to_owned()))
        .collect();

    sorted_results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1) // Sort by frequency (descending)
            .unwrap_or(std::cmp::Ordering::Equal) // Handle NaN gracefully
            .then_with(|| a.0.cmp(&b.0)) // Secondary sort by ticker symbol (ascending)
    });

    sorted_results
}
