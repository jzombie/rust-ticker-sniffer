use std::collections::{HashMap, HashSet};

use crate::utils::count_ticker_symbol_frequencies;
use crate::{CompanyTokenMapper, TokenParityState};
use ticker_sniffer_common_lib::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbol,
    TickerSymbolFrequencyMap, TickerSymbolTokenId, Token, TokenId,
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

    pub fn calc_exact_ticker_symbol_match_ratio(top_token_range_states: &[TokenRangeState]) -> f32 {
        if top_token_range_states.is_empty() {
            return 1.0;
        }

        let (exact_matches, total) =
            top_token_range_states
                .iter()
                .fold((0, 0), |(exact_matches, total), state| {
                    (
                        exact_matches
                            + if state.is_matched_on_ticker_symbol == Some(true) {
                                1
                            } else {
                                0
                            },
                        total + 1,
                    )
                });

        let ratio_exact_matches = if total > 0 {
            exact_matches as f32 / total as f32
        } else {
            0.0
        };

        ratio_exact_matches
    }

    /// Given a vector of token range states, this counts the number of symbols iwth unique query token indices
    pub fn count_token_range_ticker_symbol_frequencies(
        range_states: &[TokenRangeState],
    ) -> TickerSymbolFrequencyMap {
        // Step 1: Deduplicate query token indices for each ticker symbol
        let mut ticker_symbol_query_indices: HashMap<TickerSymbol, HashSet<Vec<QueryTokenIndex>>> =
            HashMap::new();

        for state in range_states {
            let ticker_symbol = state.ticker_symbol.clone();
            let query_token_indices = state.query_token_indices.clone();

            ticker_symbol_query_indices
                .entry(ticker_symbol)
                .or_insert_with(HashSet::new)
                .insert(query_token_indices);
        }

        // Step 2: Flatten deduplicated indices into a list of ticker symbols
        let ticker_symbols: Vec<TickerSymbol> = ticker_symbol_query_indices
            .into_iter()
            .flat_map(|(ticker_symbol, query_index_sets)| {
                query_index_sets
                    .into_iter()
                    .map(move |_| ticker_symbol.clone())
            })
            .collect();

        // Step 3: Count frequencies using the wrapper
        count_ticker_symbol_frequencies(&ticker_symbols)
    }

    pub fn collect_top_range_states(
        query_text_doc_token_ids: &[TokenId],
        token_range_states: &[TokenRangeState],
    ) -> Vec<TokenRangeState> {
        let mut top_range_states_map: Vec<Vec<&TokenRangeState>> =
            vec![Vec::new(); query_text_doc_token_ids.len()];

        for token_range_state in token_range_states {
            for &query_token_idx in &token_range_state.query_token_indices {
                if let Some(range_score) = token_range_state.range_score {
                    if top_range_states_map[query_token_idx].is_empty() {
                        // Initialize with the current range state if no state exists
                        top_range_states_map[query_token_idx].push(token_range_state);
                    } else {
                        // TODO: Don't use unwrap
                        // Check the score of the existing states
                        let existing_score = top_range_states_map[query_token_idx][0]
                            .range_score
                            .unwrap();

                        if range_score > existing_score {
                            // Replace with a new top scorer
                            top_range_states_map[query_token_idx].clear();
                            top_range_states_map[query_token_idx].push(token_range_state);
                        } else if (range_score - existing_score).abs() < f32::EPSILON {
                            // Add to the top scorers in case of a tie
                            top_range_states_map[query_token_idx].push(token_range_state);
                        }
                    }
                }
            }
        }

        // Collect only the valid range states
        let top_range_states: Vec<&TokenRangeState> = top_range_states_map
            .into_iter()
            .flat_map(|states| states) // Flatten the vector of vectors
            .collect();

        top_range_states.into_iter().cloned().collect()
    }

    /// Determines the highest scores which map to each filtered token index.
    pub fn assign_token_range_scores(
        query_text_doc_token_ids: &[TokenId],
        token_range_states: &mut [TokenRangeState],
    ) {
        for (query_token_idx, _query_token_id) in query_text_doc_token_ids.iter().enumerate() {
            // Initialize a map to store scores for this token
            let mut token_scores: HashMap<Token, f32> = HashMap::new();

            // Iterate over all token range states
            for token_range_state in &mut *token_range_states {
                // Check if the current filtered token ID is part of the filtered token IDs in the range state
                if token_range_state
                    .query_token_indices
                    .contains(&query_token_idx)
                {
                    let score = token_range_state.company_token_coverage
                        // Increase score by continuity
                        // TODO: Weight this accordingly
                        + token_range_state
                            .query_token_indices
                            .iter()
                            .position(|&x| x == query_token_idx)
                            .map(|idx| idx as f32)
                            .unwrap_or(0.0);

                    // TODO: Remove
                    // println!("score: {:?}, state: {:?}", score, token_range_state);

                    token_range_state.range_score = Some(score);

                    // Update the score map for this ticker symbol
                    token_scores
                        .entry(token_range_state.ticker_symbol.clone())
                        .and_modify(|existing_score| {
                            *existing_score = (*existing_score).max(score);
                        })
                        .or_insert(score);
                }
            }

            // Filter token_scores to retain only the highest scores
            if !token_scores.is_empty() {
                let max_score = token_scores.values().cloned().fold(f32::MIN, f32::max); // Find the maximum score
                token_scores.retain(|_, &mut score| score == max_score); // Retain entries with the highest score
            }
        }
    }

    /// The returned vector represents unique token range states.
    pub fn collect_token_range_states(
        company_token_mapper: &CompanyTokenMapper,
        potential_token_id_sequences: &HashMap<
            TickerSymbolTokenId,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        >,
        token_parity_states: &[TokenParityState],
    ) -> Vec<TokenRangeState> {
        let mut token_range_states: Vec<TokenRangeState> = Vec::new();

        for (ticker_symbol_token_id, _) in potential_token_id_sequences {
            // TODO: This is the token not the symbol!
            // TODO: Don't use unwrap here
            let ticker_symbol = company_token_mapper
                .get_ticker_symbol_by_token_id(&ticker_symbol_token_id)
                .unwrap();

            // TODO: Remove
            // println!("ticker symbol: {}", ticker_symbol);

            // Initialize state variables to track the last indices for continuity checks.
            let mut last_company_sequence_idx = usize::MAX - 1;
            let mut last_company_sequence_token_idx = usize::MAX - 1;
            let mut last_query_token_idx = usize::MAX - 1;

            // Indicates whether we are starting a new subsequence.
            // A subsequence is a group of contiguous tokens from the query and company sequences
            // that belong to the same ticker symbol and are aligned in both sequences.
            let mut is_new_sub_sequence = false;

            // Current token range state being constructed.
            let mut token_range_state: Option<TokenRangeState> = None;

            for token_parity_state in token_parity_states {
                if token_parity_state.ticker_symbol_token_id != *ticker_symbol_token_id {
                    last_company_sequence_idx = usize::MAX - 1;
                    last_company_sequence_token_idx = usize::MAX - 1;
                    last_query_token_idx = usize::MAX - 1;

                    continue;
                }

                is_new_sub_sequence = token_parity_state.company_sequence_token_idx == 0
                    || last_company_sequence_idx != token_parity_state.company_sequence_idx
                    || token_parity_state.company_sequence_token_idx
                        != last_company_sequence_token_idx + 1
                    || token_parity_state.query_token_idx != last_query_token_idx + 1;

                if is_new_sub_sequence {
                    // Finalize previous batch, if exists
                    if let Some(ref mut token_range_state) = token_range_state {
                        if !token_range_state.is_finalized {
                            token_range_state.finalize();

                            if !token_range_state.query_token_indices.is_empty() {
                                token_range_states.push(token_range_state.clone());
                            }
                        }
                    }

                    token_range_state = Some(TokenRangeState::new(
                        ticker_symbol.to_string(),
                        *ticker_symbol_token_id,
                        token_parity_state.company_sequence_idx,
                        company_token_mapper
                            .get_company_token_sequence_max_length(
                                &ticker_symbol_token_id,
                                token_parity_state.company_sequence_idx,
                            )
                            // TODO: Replace with ?
                            .unwrap(),
                    ));
                }

                // Add partial state to the current token range state
                if let Some(ref mut token_range_state) = token_range_state {
                    // Only add the current token to the token range if:
                    // - It's not a new subsequence, or
                    // - It is the first token in the company sequence.
                    if !(is_new_sub_sequence && token_parity_state.company_sequence_token_idx != 0)
                    {
                        token_range_state.add_partial_state(
                            token_parity_state.query_token_idx,
                            token_parity_state.query_token_id,
                            token_parity_state.company_sequence_token_idx,
                        );
                    }
                }

                last_company_sequence_idx = token_parity_state.company_sequence_idx;
                last_company_sequence_token_idx = token_parity_state.company_sequence_token_idx;
                last_query_token_idx = token_parity_state.query_token_idx;

                is_new_sub_sequence = false;
            }

            // Finalize previous batch, if exists
            if let Some(ref mut token_range_state) = token_range_state {
                if !token_range_state.is_finalized {
                    token_range_state.finalize();

                    if !token_range_state.query_token_indices.is_empty() {
                        token_range_states.push(token_range_state.clone());
                    }
                }
            }
        }

        TokenRangeState::get_unique(&token_range_states)
    }

    // TODO: Make this non-public if possible
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
