use crate::types::{CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap, Token, TokenId};
use crate::utils::dedup_vector;
use crate::Error;
use crate::TokenMapper;
use crate::Tokenizer;
use std::collections::{HashMap, HashSet};

type QueryTokenIndex = usize;
type CompanySequenceIndex = usize;
type CompanySequenceTokenIndex = usize;

pub struct CompanyTokenProcessorConfig {
    // TODO: Rename
    pub threshold_ratio_exact_matches: f32,
}

pub struct CompanyTokenProcessor<'a> {
    config: &'a CompanyTokenProcessorConfig,
    company_symbol_list: &'a CompanySymbolList,
    token_mapper: TokenMapper,
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    // TODO: Compute these during compile time, not runtime
    company_token_sequences: HashMap<TickerSymbol, Vec<Vec<TokenId>>>,
    company_reverse_token_map: HashMap<TokenId, Vec<TickerSymbol>>,
}

#[derive(Debug, Clone)]
struct TokenParityState {
    ticker_symbol: TickerSymbol,
    query_token_idx: QueryTokenIndex,
    query_token_id: TokenId,
    company_sequence_idx: CompanySequenceIndex,
    company_sequence_token_idx: CompanySequenceTokenIndex,
}

#[derive(Debug, Clone)]
struct TokenRangeState {
    ticker_symbol: TickerSymbol,
    ticker_symbol_token_id: TokenId,
    is_matched_on_ticker_symbol: Option<bool>,
    // TODO: Track TD-IDF scores of query tokens in relation to the query itself?
    // TODO: Track vector_similarity_state_indices?
    // vector_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    query_token_indices: Vec<QueryTokenIndex>,
    query_text_doc_token_ids: Vec<TokenId>,
    company_sequence_idx: CompanySequenceIndex,
    company_sequence_token_indices: Vec<CompanySequenceTokenIndex>,
    company_sequence_max_length: usize,
    company_token_coverage: f32,
    range_score: Option<f32>,
    is_finalized: bool,
}

impl TokenRangeState {
    fn new(
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

    fn add_partial_state(
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

    fn finalize(&mut self) {
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
}

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(
        config: &'a CompanyTokenProcessorConfig,
        company_symbol_list: &'a CompanySymbolList,
    ) -> Self {
        let mut instance = CompanyTokenProcessor {
            config,
            company_symbol_list,
            token_mapper: TokenMapper::new(),
            ticker_symbol_tokenizer: Tokenizer::ticker_symbol_parser(),
            text_doc_tokenizer: Tokenizer::text_doc_parser(),
            company_token_sequences: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        };

        instance.ingest_company_tokens();

        instance
    }

    pub fn process_text_doc(&mut self, text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
        // Tokenize the input text
        println!("Tokenizing...");
        let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(text);
        let text_doc_tokens = self.text_doc_tokenizer.tokenize(text);

        println!("Gathering filtered tokens...");
        let (query_text_doc_token_ids, query_ticker_symbol_token_ids) =
            self.get_filtered_query_token_ids(&ticker_symbol_tokens, &text_doc_tokens)?;

        // Identify token ID sequences which start with the first token of a company token sequence
        println!("Identifying token ID sequences...");
        let potential_token_id_sequences =
            self.get_potential_token_sequences(&query_text_doc_token_ids);

        // Aggregate token parity states
        println!("Collecting token parity states...");
        let token_parity_states = self
            .collect_token_parity_states(&query_text_doc_token_ids, &potential_token_id_sequences);

        // Determine range states
        println!("Collecting token range states...");
        let mut token_range_states =
            self.collect_token_range_states(&potential_token_id_sequences, &token_parity_states);

        // Assign scores to the range states
        println!("Assigning range scores...");
        self.assign_token_range_scores(&query_text_doc_token_ids, &mut token_range_states);

        // Collect top range states
        println!("Collecting top range states...");
        let top_range_states =
            self.collect_top_range_states(&query_text_doc_token_ids, &token_range_states);

        // TODO: Remove
        // Debug: Print the top range states
        // println!("Top Range States for Each Query Token Index:");
        // for state in &top_range_states {
        //     println!("{:?}", state);
        // }

        // Used to determine whether to explicitly parse out symbols which may also be stop words, based on
        // percentage of symbols to company names in the doc (for instance, determine if "A" should be parsed
        // as a symbol)
        let ratio_exact_matches = self.calc_exact_ticker_symbol_match_ratio(&top_range_states);

        // Keep track of number of occurrences, per extracted symbol, for context stats
        let text_doc_ticker_frequencies =
            self.count_token_range_ticker_symbol_frequencies(&top_range_states);

        let query_ticker_symbols: Vec<TickerSymbol> = self
            .token_mapper
            .get_tokens_by_ids(&query_ticker_symbol_token_ids)
            .into_iter()
            .filter_map(|option| option)
            .collect();

        let unique_query_ticker_symbols = dedup_vector(&query_ticker_symbols);

        let unique_text_doc_ticker_symbols: Vec<TickerSymbol> =
            text_doc_ticker_frequencies.keys().cloned().collect();

        println!(
            "query_text_doc_token_ids: {:?}, query_text_doc_tokens: {:?}, query_ticker_symbols: {:?}, unique_query_ticker_symbols: {:?}, text_doc_ticker_frequencies: {:?}, ratio_exact_matches: {}, match_threshold: {}",
            query_text_doc_token_ids, self.token_mapper.get_tokens_by_ids(&query_text_doc_token_ids), &query_ticker_symbols, &unique_query_ticker_symbols, text_doc_ticker_frequencies, ratio_exact_matches, self.config.threshold_ratio_exact_matches
        );

        // TODO: Filter out token range states less than a minimum score threshold (is this still necessary?)

        let query_tickers_not_in_text_doc: Vec<TickerSymbol> = unique_query_ticker_symbols
            .clone()
            .into_iter()
            .filter(|symbol| !unique_text_doc_ticker_symbols.contains(symbol))
            .collect();

        let query_ticker_frequencies =
            self.count_ticker_symbol_frequencies(&query_tickers_not_in_text_doc);

        let combined_ticker_frequencies = self.combine_ticker_symbol_frequencies(&[
            text_doc_ticker_frequencies.clone(),
            query_ticker_frequencies.clone(),
        ]);

        println!(
            "unique_text_doc_ticker_symbols: {:?}, unique_query_ticker_symbols: {:?}, query_tickers_not_in_text_doc: {:?}, text_doc_ticker_frequencies: {:?}, query_ticker_frequencies: {:?}, combined_ticker_frequencies: {:?}",
            unique_text_doc_ticker_symbols, unique_query_ticker_symbols, query_tickers_not_in_text_doc, text_doc_ticker_frequencies, query_ticker_frequencies, combined_ticker_frequencies
        );

        Ok(combined_ticker_frequencies)
    }

    fn combine_ticker_symbol_frequencies(
        &self,
        ticker_symbol_frequency_hash_maps: &[TickerSymbolFrequencyMap],
    ) -> TickerSymbolFrequencyMap {
        let mut combined_ticker_frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

        for frequency_hash_map in ticker_symbol_frequency_hash_maps {
            for (ticker_symbol, frequency) in frequency_hash_map {
                *combined_ticker_frequencies
                    .entry(ticker_symbol.clone())
                    .or_insert(0) += frequency;
            }
        }

        combined_ticker_frequencies
    }

    fn count_ticker_symbol_frequencies(
        &self,
        ticker_symbols: &[TickerSymbol],
    ) -> HashMap<TickerSymbol, usize> {
        let mut frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

        for ticker_symbol in ticker_symbols {
            *frequencies.entry(ticker_symbol.clone()).or_insert(0) += 1;
        }

        frequencies
    }

    /// Given a vector of token range states, this counts the number of symbols iwth unique query token indices
    fn count_token_range_ticker_symbol_frequencies(
        &self,
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
        self.count_ticker_symbol_frequencies(&ticker_symbols)
    }

    fn calc_exact_ticker_symbol_match_ratio(
        &self,
        top_token_range_states: &[TokenRangeState],
    ) -> f32 {
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

    fn get_ticker_symbol_token_id(&self, ticker_symbol: &TickerSymbol) -> Result<TokenId, String> {
        match self.company_token_sequences.get(ticker_symbol) {
            Some(sequences) => {
                if let Some(sequence) = sequences.first() {
                    if sequence.len() > 1 {
                        // Return an error if the first sequence has more than one token ID
                        return Err(format!(
                            "Error: First token ID sequence for ticker '{}' has more than one element.",
                            ticker_symbol
                        ));
                    }
                    // Return the first token ID if available
                    sequence.first().cloned().ok_or_else(|| {
                        format!(
                            "Error: First sequence for ticker '{}' is empty.",
                            ticker_symbol
                        )
                    })
                } else {
                    // Return an error if no sequences exist for the ticker
                    Err(format!(
                        "Error: No sequences found for ticker '{}'.",
                        ticker_symbol
                    ))
                }
            }
            None => Err(format!(
                "Error: Ticker '{}' not found in company token sequences.",
                ticker_symbol
            )),
        }
    }

    fn get_company_token_sequence_max_length(
        &self,
        ticker_symbol: &TickerSymbol,
        company_sequence_idx: CompanySequenceIndex,
    ) -> Option<usize> {
        self.company_token_sequences
            .get(ticker_symbol)
            .and_then(|seq| seq.get(company_sequence_idx).map(|s| s.len()))
    }
    #[allow(dead_code)]
    /// For debugging purposes
    fn display_company_tokens(&self, ticker_symbol: &TickerSymbol) {
        if let Some(company_token_sequences) = self.company_token_sequences.get(ticker_symbol) {
            for company_token_sequence in company_token_sequences {
                println!(
                    "{:?}",
                    self.token_mapper.get_tokens_by_ids(company_token_sequence)
                );
            }
        } else {
            println!("No tokens found for ticker symbol: {}", ticker_symbol);
        }
    }

    /// Ingests tokens from the company symbol list
    fn ingest_company_tokens(&mut self) {
        self.company_token_sequences.clear();
        self.company_reverse_token_map.clear();

        for (ticker_symbol, company_name, alt_company_names) in self.company_symbol_list {
            // let company_name_key = company_name.clone().unwrap();

            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                let ticker_symbol_token_id = self.token_mapper.upsert_token(&ticker_symbol_token);
                all_company_name_token_ids.push(vec![ticker_symbol_token_id]);

                // Populate reverse map
                self.company_reverse_token_map
                    .entry(ticker_symbol_token_id)
                    .or_insert_with(Vec::new)
                    .push(ticker_symbol.clone());
            }

            if let Some(company_name) = company_name {
                let company_name_token_ids = self.process_company_name_tokens(&company_name);
                all_company_name_token_ids.push(company_name_token_ids.clone());

                // Populate reverse map
                for token_id in company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id.clone())
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol.clone());
                }
            }

            // Process alternate company names
            for alt_company_name in alt_company_names {
                let alt_company_name_token_ids =
                    self.process_company_name_tokens(&alt_company_name);
                all_company_name_token_ids.push(alt_company_name_token_ids.clone());

                // Populate reverse map
                for token_id in alt_company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol.clone());
                }
            }

            // Insert the collected token IDs into the map
            self.company_token_sequences
                .entry(ticker_symbol.clone())
                .or_insert_with(Vec::new)
                .extend(all_company_name_token_ids);
        }
    }

    /// Helper method for per-company token ingestion
    fn process_company_name_tokens(&mut self, company_name: &str) -> Vec<TokenId> {
        let company_name_tokens = self.text_doc_tokenizer.tokenize(&company_name);
        let mut company_name_token_ids = Vec::new();
        for token in company_name_tokens {
            let token_id = self.token_mapper.upsert_token(&token);
            company_name_token_ids.push(token_id);
        }

        company_name_token_ids
    }

    fn get_filtered_query_token_ids(
        &self,
        ticker_symbol_tokens: &Vec<Token>,
        text_doc_tokens: &Vec<Token>,
    ) -> Result<(Vec<TokenId>, Vec<TokenId>), Error> {
        if text_doc_tokens.is_empty() {
            // Return an error if no tokens are found
            return Err(Error::TokenFilterError(
                "No tokens found in the text document.".to_string(),
            ));
        }

        // Get the filtered token IDs (IDs present in the TokenMapper)
        let query_text_doc_token_ids = self
            .token_mapper
            .get_filtered_token_ids(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        let query_ticker_symbol_token_ids = self
            .token_mapper
            .get_filtered_token_ids(ticker_symbol_tokens.iter().map(|s| s.as_str()).collect());

        // Return the filtered token IDs wrapped in `Ok`
        Ok((query_text_doc_token_ids, query_ticker_symbol_token_ids))
    }

    fn get_potential_token_sequences(
        &self,
        query_text_doc_token_ids: &[TokenId],
    ) -> HashMap<TickerSymbol, Vec<(CompanySequenceIndex, Vec<TokenId>)>> {
        let mut potential_token_id_sequences: HashMap<
            TickerSymbol,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        > = HashMap::new();

        for query_token_id in query_text_doc_token_ids {
            if let Some(possible_ticker_symbols) =
                self.company_reverse_token_map.get(query_token_id)
            {
                for ticker_symbol in possible_ticker_symbols {
                    if let Some(company_name_variations_token_ids_list) =
                        self.company_token_sequences.get(ticker_symbol)
                    {
                        for (company_sequence_idx, company_name_variations_token_ids) in
                            company_name_variations_token_ids_list.iter().enumerate()
                        {
                            if company_name_variations_token_ids.is_empty() {
                                continue;
                            }

                            let company_name_first_token_id = company_name_variations_token_ids[0];

                            if *query_token_id == company_name_first_token_id {
                                // Add or update the hashmap entry for this ticker_symbol
                                potential_token_id_sequences
                                    .entry(ticker_symbol.clone())
                                    .or_insert_with(Vec::new) // Create an empty Vec if the key doesn't exist
                                    .retain(|(existing_idx, existing_vec)| {
                                        *existing_idx != company_sequence_idx
                                            || *existing_vec != *company_name_variations_token_ids
                                    }); // Remove duplicates

                                // TODO: Don't use unwrap
                                if !potential_token_id_sequences
                                    .get(&ticker_symbol.clone())
                                    .unwrap()
                                    .iter()
                                    .any(|(existing_idx, existing_vec)| {
                                        *existing_idx == company_sequence_idx
                                            && *existing_vec == *company_name_variations_token_ids
                                    })
                                {
                                    potential_token_id_sequences
                                        .get_mut(&ticker_symbol.to_string())
                                        .unwrap()
                                        .push((
                                            company_sequence_idx,
                                            company_name_variations_token_ids.clone(),
                                        ));
                                }
                            }
                        }
                    }
                }
            }
        }

        potential_token_id_sequences
    }

    fn collect_token_parity_states(
        &self,
        query_text_doc_token_ids: &[TokenId],
        potential_token_id_sequences: &HashMap<
            TickerSymbol,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        >,
    ) -> Vec<TokenParityState> {
        let mut token_parity_states = Vec::new();

        for (ticker_symbol, company_token_sequences) in potential_token_id_sequences {
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
                                ticker_symbol: ticker_symbol.to_string(),
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
                &a.ticker_symbol,
                a.company_sequence_idx,
                a.query_token_idx,
                a.company_sequence_token_idx,
            )
                .cmp(&(
                    &b.ticker_symbol,
                    b.company_sequence_idx,
                    b.query_token_idx,
                    b.company_sequence_token_idx,
                ))
        });

        token_parity_states
    }

    fn collect_token_range_states(
        &self,
        potential_token_id_sequences: &HashMap<
            TickerSymbol,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        >,
        token_parity_states: &[TokenParityState],
    ) -> Vec<TokenRangeState> {
        let mut token_range_states: Vec<TokenRangeState> = Vec::new();

        for (ticker_symbol, _) in potential_token_id_sequences {
            // TODO: Don't use unwrap here
            let ticker_symbol_token_id = self.get_ticker_symbol_token_id(ticker_symbol).unwrap();

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
                if token_parity_state.ticker_symbol != *ticker_symbol {
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
                        ticker_symbol_token_id,
                        token_parity_state.company_sequence_idx,
                        self.get_company_token_sequence_max_length(
                            ticker_symbol,
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

        token_range_states
    }

    /// Determines the highest scores which map to each filtered token index.
    fn assign_token_range_scores(
        &self,
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

    fn collect_top_range_states(
        &self,
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
}
