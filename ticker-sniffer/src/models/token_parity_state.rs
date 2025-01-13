use std::collections::HashMap;
use ticker_sniffer_common_lib::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbolTokenId, TokenId,
};

#[derive(Debug, Clone)]
pub struct TokenParityState {
    pub ticker_symbol_token_id: TickerSymbolTokenId,
    pub query_token_idx: QueryTokenIndex,
    pub query_token_id: TokenId,
    pub company_sequence_idx: CompanySequenceIndex,
    pub company_sequence_token_idx: CompanySequenceTokenIndex,
}

impl TokenParityState {
    pub fn collect_token_parity_states(
        query_text_doc_token_ids: &[TokenId],
        potential_token_id_sequences: &HashMap<
            TickerSymbolTokenId,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        >,
    ) -> Vec<TokenParityState> {
        let mut token_parity_states = Vec::new();

        for (ticker_symbol_token_id, company_token_sequences) in potential_token_id_sequences {
            for company_sequence_tuple in company_token_sequences {
                for (query_token_idx, query_token_id) in query_text_doc_token_ids.iter().enumerate()
                {
                    let company_sequence_idx = &company_sequence_tuple.0;
                    let company_sequence_token_ids = &company_sequence_tuple.1;

                    for (company_sequence_token_idx, company_sequence_token_id) in
                        company_sequence_token_ids.iter().enumerate()
                    {
                        if company_sequence_token_id == query_token_id {
                            token_parity_states.push(TokenParityState {
                                ticker_symbol_token_id: *ticker_symbol_token_id,
                                query_token_idx,
                                query_token_id: *query_token_id,
                                company_sequence_idx: *company_sequence_idx,
                                company_sequence_token_idx,
                            });
                        }
                    }
                }
            }
        }

        // Reorder token_parity_states
        token_parity_states.sort_by(|a, b| {
            (
                &a.ticker_symbol_token_id,
                a.company_sequence_idx,
                a.query_token_idx,
                a.company_sequence_token_idx,
            )
                .cmp(&(
                    &b.ticker_symbol_token_id,
                    b.company_sequence_idx,
                    b.query_token_idx,
                    b.company_sequence_token_idx,
                ))
        });

        token_parity_states
    }
}
