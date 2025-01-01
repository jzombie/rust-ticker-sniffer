use crate::types::CompanySymbolList;
use crate::types::TickerSymbol;
use crate::TokenMapper;
use crate::Tokenizer;
use std::collections::HashMap;

pub struct CompanyTokenProcessor<'a> {
    company_symbol_list: &'a CompanySymbolList,
    token_mapper: TokenMapper,
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    // TODO: Use TokenId instead of usize
    company_token_sequences: HashMap<TickerSymbol, Vec<Vec<usize>>>,
    company_reverse_token_map: HashMap<usize, Vec<TickerSymbol>>,
}

#[derive(Debug, Clone)]
struct TokenParityState {
    ticker_symbol: TickerSymbol,
    query_token_idx: usize,
    query_token_id: usize,
    company_sequence_idx: usize,
    company_sequence_token_idx: usize,
}

#[derive(Debug, Clone)]
struct TokenRangeState {
    ticker_symbol: TickerSymbol,
    // TODO: Track TD-IDF scores of query tokens in relation to the query itself?
    // TODO: Track vector_similarity_state_indices?
    // vector_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    query_token_indices: Vec<usize>,
    query_token_ids: Vec<usize>,
    company_sequence_idx: usize,
    company_sequence_token_indices: Vec<usize>,
    company_sequence_max_length: usize,
    company_token_coverage: f32,
    range_score: Option<f32>,
    is_finalized: bool,
}

impl TokenRangeState {
    fn new(
        ticker_symbol: TickerSymbol,
        company_sequence_idx: usize,
        company_sequence_max_length: usize,
    ) -> Self {
        TokenRangeState {
            ticker_symbol,
            query_token_indices: vec![],
            query_token_ids: vec![],
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
        query_token_idx: usize,
        query_token_id: usize,
        company_sequence_token_idx: usize,
    ) {
        self.query_token_indices.push(query_token_idx);
        self.query_token_ids.push(query_token_id);
        self.company_sequence_token_indices
            .push(company_sequence_token_idx);
    }

    fn finalize(&mut self) {
        self.update_coverage();
        self.is_finalized = true;
    }

    /// Recalculates the coverage based on the filtered indices and sequence length
    fn update_coverage(&mut self) {
        self.company_token_coverage =
            self.query_token_indices.len() as f32 / self.company_sequence_max_length as f32;
    }
}

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        let mut instance = CompanyTokenProcessor {
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

    fn get_company_token_sequence_max_length(
        &self,
        ticker_symbol: &TickerSymbol,
        company_sequence_idx: usize,
    ) -> Option<usize> {
        // TODO: REmove
        println!(
            "{}, {:?}, {:?}",
            ticker_symbol,
            self.company_token_sequences
                .get(ticker_symbol)
                .and_then(|seq| seq.get(company_sequence_idx)),
            self.token_mapper.get_tokens_by_ids(
                self.company_token_sequences
                    .get(ticker_symbol)
                    .and_then(|seq| seq.get(company_sequence_idx))
                    .unwrap()
            )
        );

        self.company_token_sequences
            .get(ticker_symbol)
            .and_then(|seq| seq.get(company_sequence_idx).map(|s| s.len()))
    }

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
    fn process_company_name_tokens(&mut self, company_name: &str) -> Vec<usize> {
        let company_name_tokens = self.text_doc_tokenizer.tokenize(&company_name);
        let mut company_name_token_ids = Vec::new();
        for token in company_name_tokens {
            let token_id = self.token_mapper.upsert_token(&token);
            company_name_token_ids.push(token_id);
        }

        company_name_token_ids
    }

    // TODO: Use Result type for output
    pub fn process_text_doc(&mut self, text: &str) {
        // Tokenize the input text
        let text_doc_tokens = self.text_doc_tokenizer.tokenize(text);

        // TODO: Remove
        println!("{:?}", text_doc_tokens);

        if text_doc_tokens.is_empty() {
            println!("No tokens found in the text document. Exiting.");
            return;
        }

        // Get the filtered tokens (tokens present in the TokenMapper)
        let query_tokens = self
            .token_mapper
            .get_filtered_tokens(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        // Get the filtered token IDs (IDs present in the TokenMapper)
        let query_token_ids = self
            .token_mapper
            .get_filtered_token_ids(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        if query_token_ids.is_empty() {
            println!("No token IDs found in the document.");
            return;
        }

        // Identify token ID sequences which start with the first token of a company token sequence
        let mut potential_token_id_sequences: HashMap<TickerSymbol, Vec<(usize, Vec<usize>)>> =
            HashMap::new();

        for query_token_id in &query_token_ids {
            if let Some(possible_ticker_symbols) =
                self.company_reverse_token_map.get(query_token_id)
            {
                for ticker_symbol in possible_ticker_symbols {
                    if let Some(company_name_variations_token_ids_list) =
                        self.company_token_sequences.get(ticker_symbol)
                    {
                        for (company_token_sequence_idx, company_name_variations_token_ids) in
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
                                        *existing_idx != company_token_sequence_idx
                                            || *existing_vec != *company_name_variations_token_ids
                                    }); // Remove duplicates

                                if !potential_token_id_sequences
                                    .get(&ticker_symbol.clone())
                                    .unwrap()
                                    .iter()
                                    .any(|(existing_idx, existing_vec)| {
                                        *existing_idx == company_token_sequence_idx
                                            && *existing_vec == *company_name_variations_token_ids
                                    })
                                {
                                    potential_token_id_sequences
                                        .get_mut(&ticker_symbol.to_string())
                                        .unwrap()
                                        .push((
                                            company_token_sequence_idx,
                                            company_name_variations_token_ids.clone(),
                                        ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Aggregate token parity states
        let mut token_parity_states = Vec::new();
        for (ticker_symbol, company_token_sequences) in &potential_token_id_sequences {
            for company_sequence_tuple in company_token_sequences {
                for (query_token_idx, query_token_id) in query_token_ids.iter().enumerate() {
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

        for token_parity_state in &token_parity_states {
            println!(
                "{:?}, {:?}",
                token_parity_state,
                self.token_mapper
                    .get_token_by_id(token_parity_state.query_token_id)
            );
        }

        // Determine range states
        let mut token_range_states: Vec<TokenRangeState> = Vec::new();

        for (ticker_symbol, _) in &potential_token_id_sequences {
            let mut last_company_sequence_idx = usize::MAX - 1;
            let mut last_company_sequence_token_idx = usize::MAX - 1;
            let mut last_query_token_idx = usize::MAX - 1;

            let mut is_new_sub_sequence = false;

            let mut token_range_state: Option<TokenRangeState> = None;

            for token_parity_state in &token_parity_states {
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
                        token_parity_state.company_sequence_idx,
                        self.get_company_token_sequence_max_length(
                            ticker_symbol,
                            token_parity_state.company_sequence_idx,
                        )
                        // TODO: Replace with ?
                        .unwrap(),
                    ));
                }

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

            // println!("==========");
        }

        // Print the range states for debugging
        for token_range_state in &token_range_states {
            // if token_range_state.company_token_coverage >= 0.5 {
            println!(
                "{:?}, Tokens: {:?}",
                token_range_state,
                self.token_mapper
                    .get_tokens_by_ids(&token_range_state.query_token_ids)
            );
            // }
        }

        // TODO: Determine the highest scores which map to each filtered token index
        // let mut query_token_idx_top_ranges: HashMap<usize, Vec<TokenRangeState>> = HashMap::new();
        for (query_token_idx, query_token_id) in query_token_ids.iter().enumerate() {
            // Initialize a map to store scores for this token
            let mut token_scores: HashMap<String, f32> = HashMap::new();

            // Iterate over all token range states
            for token_range_state in &mut token_range_states {
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

                    // query_token_idx_top_ranges
                    //     .entry(query_token_idx)
                    //     .or_insert_with(Vec::new)
                    //     .push(token_range_state.clone());
                }
            }

            // Filter token_scores to retain only the highest scores
            if !token_scores.is_empty() {
                let max_score = token_scores.values().cloned().fold(f32::MIN, f32::max); // Find the maximum score
                token_scores.retain(|_, &mut score| score == max_score); // Retain entries with the highest score
            }

            // Debug output for the current token index
            println!(
                "Query Token Index: {}, Token ID: {}, Token: {:?}, Scores: {:?}",
                query_token_idx,
                query_token_id,
                self.token_mapper.get_token_by_id(*query_token_id),
                token_scores
            );
        }

        // for (query_token_idx, token_range_states) in &query_token_idx_top_ranges {
        //     for token_range_state in token_range_states {
        //         println!(
        //             "query tok. idx: {:?}, {:?}",
        //             query_token_idx, token_range_state
        //         );
        //     }
        // }

        // TODO: Remove
        self.display_company_tokens(&"GJO".to_string());

        // Convert the scores HashMap into a sorted Vec
        // let mut sorted_scores: Vec<(String, f32)> = scores.clone().into_iter().collect();
        // sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sort descending by score

        // TODO: Remove
        println!("Text doc tokens: {:?}", text_doc_tokens);
        println!("Filtered tokens: {:?}", query_tokens);
        println!("Query token IDs: {:?}", query_token_ids);
        // println!("Possible matches: {:?}", potential_token_id_sequences);
        // println!("Scores: {:?}", scores);
    }
}
