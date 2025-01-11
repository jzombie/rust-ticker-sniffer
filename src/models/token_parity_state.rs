use crate::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbol, TokenId,
};

#[derive(Debug, Clone)]
pub struct TokenParityState {
    pub ticker_symbol: TickerSymbol,
    pub query_token_idx: QueryTokenIndex,
    pub query_token_id: TokenId,
    pub company_sequence_idx: CompanySequenceIndex,
    pub company_sequence_token_idx: CompanySequenceTokenIndex,
}
