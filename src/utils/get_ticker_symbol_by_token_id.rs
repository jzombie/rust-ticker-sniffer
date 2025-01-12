use crate::types::{ReverseTickerSymbolMap, TickerSymbol, TokenId};

// TODO: Use actual error type (or return option type)
pub fn get_ticker_symbol_by_token_id<'a>(
    reverse_ticker_symbol_map: &'a ReverseTickerSymbolMap,
    token_id: &'a TokenId,
) -> Result<&'a TickerSymbol, String> {
    match reverse_ticker_symbol_map.get(token_id) {
        Some(ticker_symbol) => Ok(ticker_symbol),
        None => Err("Could not obtain token id".to_string()),
    }
}
