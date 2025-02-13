use crate::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbolTokenId, TokenId,
};
use std::collections::HashMap;

/// Represents the parity between tokens in the query document and a company's token sequence.
///
/// This struct is used to track the relationship between tokens from the input query text
/// and corresponding tokens in a company's token sequences for a specific ticker symbol.
#[derive(Debug, Clone)]
pub struct TokenParityState {
    /// The unique token ID associated with a ticker symbol.
    pub ticker_symbol_token_id: TickerSymbolTokenId,

    /// The index of the query token in the query text document.
    pub query_token_idx: QueryTokenIndex,

    /// The unique token ID of the query token.
    pub query_token_id: TokenId,

    /// The index of the sequence for the company associated with the ticker symbol.
    pub company_sequence_idx: CompanySequenceIndex,

    /// The index of the token in the company's token sequence.
    pub company_sequence_token_idx: CompanySequenceTokenIndex,
}

impl TokenParityState {
    /// Collects token parity states by identifying matches between query tokens and company tokens.
    ///
    /// This function iterates through potential token sequences from company data and matches
    /// them against the token IDs from the query text. For every match, it generates a
    /// `TokenParityState` instance that captures the alignment of tokens between the query
    /// and the company's token sequence.
    ///
    /// # Arguments
    /// * `query_text_doc_token_ids` - A slice of token IDs from the query text document.
    /// * `potential_token_id_sequences` - A map where:
    ///   - The key is the `TickerSymbolTokenId` for a company.
    ///   - The value is a vector of tuples, where each tuple contains:
    ///     - `CompanySequenceIndex`: The index of the sequence.
    ///     - `Vec<TokenId>`: The token IDs in the company's sequence.
    ///
    /// # Returns
    /// * A vector of `TokenParityState` objects, sorted by:
    ///   - `ticker_symbol_token_id`
    ///   - `company_sequence_idx`
    ///   - `query_token_idx`
    ///   - `company_sequence_token_idx`
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
