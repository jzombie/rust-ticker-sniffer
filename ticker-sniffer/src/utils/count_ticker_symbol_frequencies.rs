use std::collections::HashMap;
use ticker_sniffer_common_lib::types::TickerSymbol;

pub fn count_ticker_symbol_frequencies(
    ticker_symbols: &[TickerSymbol],
) -> HashMap<TickerSymbol, usize> {
    let mut frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

    for ticker_symbol in ticker_symbols {
        *frequencies.entry(ticker_symbol.clone()).or_insert(0) += 1;
    }

    frequencies
}
