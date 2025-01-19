use std::collections::{HashMap, HashSet};

use crate::types::{
    CompanySequenceIndex, CompanySequenceTokenIndex, QueryTokenIndex, TickerSymbol,
    TickerSymbolFrequencyMap, TickerSymbolTokenId, Token, TokenId,
};
use crate::utils::count_ticker_symbol_frequencies;
use crate::{CompanyTokenMapper, Error, TokenParityState};

/// Represents a range of tokens associated with a specific ticker symbol.
///
/// This struct tracks information about tokens in a document and their relation
/// to a company's ticker symbol and token sequences.
#[derive(Debug, Clone)]
pub struct TokenRangeState {
    /// The ticker symbol associated with this token range.
    pub ticker_symbol: TickerSymbol,

    /// The unique token ID for the associated ticker symbol.
    pub ticker_symbol_token_id: TokenId,

    /// Indicates whether this range matches the ticker symbol exactly.
    /// If `None`, the match has not been determined yet.
    pub is_matched_on_ticker_symbol: Option<bool>,

    /// A list of indices of tokens in the query text document that are part
    /// of this range.
    pub query_token_indices: Vec<QueryTokenIndex>,

    /// A list of token IDs from the query text document that are part of
    /// this range.
    pub query_text_doc_token_ids: Vec<TokenId>,

    /// The index of the company's token sequence in the preprocessed symbol list.
    pub company_sequence_idx: CompanySequenceIndex,

    /// A list of indices of tokens in the company's sequence that are part
    /// of this range.
    pub company_sequence_token_indices: Vec<CompanySequenceTokenIndex>,

    /// The maximum length of the company's token sequence.
    pub company_sequence_max_length: usize,

    /// The ratio of tokens in the company's sequence that are covered
    /// by this range.
    pub company_token_coverage: f32,

    /// The score assigned to this range based on token alignment and coverage.
    /// If `None`, the range has not been scored yet.
    pub range_score: Option<f32>,

    /// Indicates whether the token range state has been finalized.
    pub is_collection_finalized: bool,
}

impl TokenRangeState {
    /// Creates a new `TokenRangeState` instance.
    ///
    /// # Arguments
    /// * `ticker_symbol` - The ticker symbol associated with the range state.
    /// * `ticker_symbol_token_id` - The unique token ID for the ticker symbol.
    /// * `company_sequence_idx` - The index of the company's token sequence.
    /// * `company_sequence_max_length` - The maximum length of the company's token sequence.
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
            is_collection_finalized: false,
        }
    }

    /// Adds a partial state to the current token range state.
    ///
    /// # Arguments
    /// * `query_token_idx` - The index of the query token in the text document.
    /// * `query_token_id` - The unique token ID for the query token.
    /// * `company_sequence_token_idx` - The index of the token in the company's token sequence.
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

    /// Calculates the ratio of exact ticker symbol matches in the top token range states.
    ///
    /// # Arguments
    /// * `top_token_range_states` - A slice of the top token range states to analyze.
    ///
    /// # Returns
    /// * The ratio of exact matches as a floating-point number.
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

        if total > 0 {
            exact_matches as f32 / total as f32
        } else {
            0.0
        }
    }

    /// Counts the frequencies of ticker symbols based on unique query token indices.
    ///
    /// # Arguments
    /// * `range_states` - A slice of token range states to analyze.
    ///
    /// # Returns
    /// * A map of ticker symbols to their frequency counts.
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
                .or_default()
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

    /// Collects the top range states by analyzing token scores and retaining the best matches.
    ///
    /// # Arguments
    /// * `query_text_doc_token_ids` - A slice of token IDs from the query text document.
    /// * `token_range_states` - A slice of token range states to evaluate.
    ///
    /// # Returns
    /// * A vector of the top range states.
    ///
    /// # Errors
    /// * Returns an error if range scores are not properly assigned.
    pub fn collect_top_range_states(
        query_text_doc_token_ids: &[TokenId],
        token_range_states: &[TokenRangeState],
    ) -> Result<Vec<TokenRangeState>, Error> {
        let mut top_range_states_map: Vec<Vec<&TokenRangeState>> =
            vec![Vec::new(); query_text_doc_token_ids.len()];

        for token_range_state in token_range_states {
            for &query_token_idx in &token_range_state.query_token_indices {
                if let Some(range_score) = token_range_state.range_score {
                    if top_range_states_map[query_token_idx].is_empty() {
                        // Initialize with the current range state if no state exists
                        top_range_states_map[query_token_idx].push(token_range_state);
                    } else {
                        // Check the score of the existing states
                        let existing_score = top_range_states_map[query_token_idx][0]
                            .range_score
                            .ok_or_else(|| {
                            Error::ParserError(format!(
                                "Could not check score of existing state with ticker symbol:{} ",
                                &token_range_state.ticker_symbol
                            ))
                        })?;

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
            .flatten() // Flatten the vector of vectors
            .collect();

        Ok(top_range_states.into_iter().cloned().collect())
    }

    /// Assigns scores to token range states based on their token coverage and continuity.
    ///
    /// # Arguments
    /// * `query_text_doc_token_ids` - A slice of token IDs from the query text document.
    /// * `token_range_states` - A mutable slice of token range states to assign scores to.
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
                        + token_range_state
                            .query_token_indices
                            .iter()
                            .position(|&x| x == query_token_idx)
                            .map(|idx| idx as f32)
                            .unwrap_or(0.0);

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

    /// Collects unique token range states by removing duplicates based on their key properties.
    ///
    /// # Arguments
    /// * `company_token_mapper` - A reference to the token mapper for company tokens.
    /// * `potential_token_id_sequences` - A map of potential token ID sequences.
    /// * `token_parity_states` - A slice of token parity states.
    ///
    /// # Returns
    /// * A vector of unique token range states.
    ///
    /// # Errors
    /// * Returns an error if fetching maximum sequence lengths or ticker symbols fails.
    pub fn collect_token_range_states(
        company_token_mapper: &CompanyTokenMapper,
        potential_token_id_sequences: &HashMap<
            TickerSymbolTokenId,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        >,
        token_parity_states: &[TokenParityState],
    ) -> Result<Vec<TokenRangeState>, Error> {
        let mut token_range_states: Vec<TokenRangeState> = Vec::new();

        for ticker_symbol_token_id in potential_token_id_sequences.keys() {
            let ticker_symbol =
                company_token_mapper.get_ticker_symbol_by_token_id(ticker_symbol_token_id)?;

            // Initialize state variables to track the last indices for continuity checks.
            let mut last_company_sequence_idx = usize::MAX - 1;
            let mut last_company_sequence_token_idx = usize::MAX - 1;
            let mut last_query_token_idx = usize::MAX - 1;

            // Indicates whether we are starting a new subsequence.
            // A subsequence is a group of contiguous tokens from the query and company sequences
            // that belong to the same ticker symbol and are aligned in both sequences.
            let mut is_new_sub_sequence: bool;

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
                        if !token_range_state.is_collection_finalized {
                            token_range_state.finalize_collection();

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
                                ticker_symbol_token_id,
                                token_parity_state.company_sequence_idx,
                            )
                            .ok_or_else(|| {
                                Error::ParserError(format!(
                                    "Failed to fetch max length for ticker_symbol_token_id: {:?}, company_sequence_idx: {}",
                                    ticker_symbol_token_id,
                                    token_parity_state.company_sequence_idx
                                ))
                            })?,
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
            }

            // Finalize previous batch, if exists
            if let Some(ref mut token_range_state) = token_range_state {
                if !token_range_state.is_collection_finalized {
                    token_range_state.finalize_collection();

                    if !token_range_state.query_token_indices.is_empty() {
                        token_range_states.push(token_range_state.clone());
                    }
                }
            }
        }

        Ok(TokenRangeState::get_unique(&token_range_states))
    }

    /// Finalizes the collection of a token range state by calculating its coverage
    /// and determining if it exactly matches a ticker symbol.
    fn finalize_collection(&mut self) {
        self.update_coverage();

        self.is_matched_on_ticker_symbol = Some(
            self.query_text_doc_token_ids.len() == 1
                && self.query_text_doc_token_ids[0] == self.ticker_symbol_token_id,
        );

        self.is_collection_finalized = true;
    }

    /// Updates the coverage of a token range state based on its token indices and sequence length.
    fn update_coverage(&mut self) {
        self.company_token_coverage =
            self.query_token_indices.len() as f32 / self.company_sequence_max_length as f32;
    }

    /// Filters and returns a vector of unique token range states by deduplication.
    ///
    /// # Arguments
    /// * `token_range_states` - A slice of token range states to deduplicate.
    ///
    /// # Returns
    /// * A vector of unique token range states.
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
