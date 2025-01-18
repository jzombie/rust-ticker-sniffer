use crate::types::TickerSymbol;
use std::collections::HashMap;

pub fn count_ticker_symbol_frequencies(
    ticker_symbols: &[TickerSymbol],
) -> HashMap<TickerSymbol, usize> {
    let mut frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

    for ticker_symbol in ticker_symbols {
        *frequencies.entry(ticker_symbol.clone()).or_insert(0) += 1;
    }

    frequencies
}
