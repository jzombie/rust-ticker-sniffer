use crate::types::{CompanySequenceIndex, CompanyTokenSequencesMap, TickerSymbol};

pub fn get_company_token_sequence_max_length(
    company_token_sequences_map: &CompanyTokenSequencesMap,
    ticker_symbol: &TickerSymbol,
    company_sequence_idx: CompanySequenceIndex,
) -> Option<usize> {
    company_token_sequences_map
        .get(ticker_symbol)
        .and_then(|seq| seq.get(company_sequence_idx).map(|s| s.len()))
}
