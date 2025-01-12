use std::collections::HashMap;

use crate::types::{
    CompanySymbolList, CompanyTokenSequencesMap, ReverseTickerSymbolMap, TickerSymbol,
    TickerSymbolMap, TokenId,
};

pub struct TickerSymbolMapper {
    pub ticker_symbol_map: TickerSymbolMap,
    pub reverse_ticker_symbol_map: ReverseTickerSymbolMap,
    // TODO: Replace tickersymbol with a token ID representing the ticker
    // symbol, and use the reverse ticker symbol map to map them back?
    pub company_token_sequences: CompanyTokenSequencesMap,
    pub company_reverse_token_map: HashMap<TokenId, Vec<TickerSymbol>>,
}

impl<'a> TickerSymbolMapper {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        TickerSymbolMapper {
            ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            reverse_ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            company_token_sequences: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.company_token_sequences.clear();
        self.company_reverse_token_map.clear();
        self.ticker_symbol_map.clear();
        self.reverse_ticker_symbol_map.clear();
    }
}
