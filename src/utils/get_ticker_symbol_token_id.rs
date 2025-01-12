use crate::types::{TickerSymbol, TickerSymbolMap, TokenId};

// TODO: Use actual error type (or return option type)
pub fn get_ticker_symbol_token_id<'a>(
    ticker_symbol_map: &'a TickerSymbolMap,
    ticker_symbol: &'a TickerSymbol,
) -> Result<&'a TokenId, String> {
    match ticker_symbol_map.get(ticker_symbol) {
        Some(token_id) => Ok(token_id),
        None => Err("Could not obtain token id".to_string()),
    }
}
