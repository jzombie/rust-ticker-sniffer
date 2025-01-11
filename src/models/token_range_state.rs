use std::collections::HashSet;

use crate::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbol, TokenId,
};

#[derive(Debug, Clone)]
pub struct TokenRangeState {
    pub ticker_symbol: TickerSymbol,
    pub ticker_symbol_token_id: TokenId,
    pub is_matched_on_ticker_symbol: Option<bool>,
    // TODO: Track TD-IDF scores of query tokens in relation to the query itself?
    // TODO: Track vector_similarity_state_indices?
    // vector_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    pub query_token_indices: Vec<QueryTokenIndex>,
    pub query_text_doc_token_ids: Vec<TokenId>,
    pub company_sequence_idx: CompanySequenceIndex,
    pub company_sequence_token_indices: Vec<CompanySequenceTokenIndex>,
    pub company_sequence_max_length: usize,
    pub company_token_coverage: f32,
    pub range_score: Option<f32>,
    // TODO: Consider renaming; I believe range scores are set after finalization
    pub is_finalized: bool,
}

impl TokenRangeState {
    pub fn new(
        ticker_symbol: TickerSymbol,
        ticker_symbol_token_id: TokenId,
        company_sequence_idx: CompanySequenceIndex,
        company_sequence_max_length: usize,
    ) -> Self {
        TokenRangeState {
            ticker_symbol,
            ticker_symbol_token_id,
            is_matched_on_ticker_symbol: None,
            query_token_indices: vec![],
            query_text_doc_token_ids: vec![],
            company_sequence_idx,
            company_sequence_token_indices: vec![],
            company_sequence_max_length,
            company_token_coverage: 0.0,
            range_score: None,
            is_finalized: false,
        }
    }

    pub fn add_partial_state(
        &mut self,
        query_token_idx: QueryTokenIndex,
        query_token_id: TokenId,
        company_sequence_token_idx: CompanySequenceTokenIndex,
    ) {
        self.query_token_indices.push(query_token_idx);
        self.query_text_doc_token_ids.push(query_token_id);
        self.company_sequence_token_indices
            .push(company_sequence_token_idx);
    }

    pub fn finalize(&mut self) {
        self.update_coverage();

        self.is_matched_on_ticker_symbol = Some(
            self.query_text_doc_token_ids.len() == 1
                && self.query_text_doc_token_ids[0] == self.ticker_symbol_token_id,
        );

        self.is_finalized = true;
    }

    /// Recalculates the coverage based on the filtered indices and sequence length
    fn update_coverage(&mut self) {
        self.company_token_coverage =
            self.query_token_indices.len() as f32 / self.company_sequence_max_length as f32;
    }

    pub fn get_unique(token_range_states: &[TokenRangeState]) -> Vec<TokenRangeState> {
        // Use a HashSet to track unique combinations of ticker_symbol and query_text_doc_token_ids
        let mut seen = HashSet::new();
        let mut unique_states = Vec::new();

        for state in token_range_states {
            // Create a tuple representing the unique key
            let unique_key = (
                &state.ticker_symbol,
                &state.query_text_doc_token_ids,
                &state.company_sequence_idx,
            );

            // Check if this combination has been seen before
            if seen.insert(unique_key) {
                unique_states.push(state.clone());
            }
        }

        unique_states
    }
}
