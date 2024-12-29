use crate::types::CompanySymbolList;
use crate::types::TickerSymbol;
use crate::TokenMapper;
use crate::Tokenizer;
use std::collections::HashMap;

pub struct CompanyTokenProcessor<'a> {
    company_symbol_list: &'a CompanySymbolList,
    token_mapper: TokenMapper,
    tokenizer: Tokenizer,
    // TODO: Use TokenId instead of usize
    company_token_sequences: HashMap<TickerSymbol, Vec<Vec<usize>>>,
    company_reverse_token_map: HashMap<usize, Vec<TickerSymbol>>,
}

#[derive(Debug, Clone)]
struct TokenParityState {
    ticker_symbol: TickerSymbol,
    filtered_token_idx: usize,
    filtered_token_id: usize,
    company_sequence_idx: usize,
    company_sequence_token_idx: usize,
}

#[derive(Debug, Clone)]
struct TokenRangeState {
    ticker_symbol: TickerSymbol,
    // TODO: Track TD-IDF scores of query tokens in relation to the query itself?
    // TODO: Track vector_similarity_state_indices?
    // vector_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    filtered_token_indices: Vec<usize>,
    filtered_token_ids: Vec<usize>,
    company_sequence_idx: usize,
    company_sequence_length: usize,
    company_sequence_token_indices: Vec<usize>,
    // company_name_token_frequencies: Vec<usize>,
    // company_name_token_frequencies_softmax: Vec<f64>,
    // company_name_token_vectors: Vec<TokenizerVectorToken>,
    // company_name_token_tf_idf_scores: Vec<f32>,
    // company_name_char_coverage: f32,
    // company_name_token_coverage: f32,
}

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        let mut instance = CompanyTokenProcessor {
            company_symbol_list,
            token_mapper: TokenMapper::new(),
            tokenizer: Tokenizer::new(),
            company_token_sequences: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        };

        instance.ingest_company_tokens();

        instance
    }

    /// Ingests tokens from the company symbol list
    fn ingest_company_tokens(&mut self) {
        self.company_token_sequences.clear();
        self.company_reverse_token_map.clear();

        for (ticker_symbol, company_name, alt_company_names) in self.company_symbol_list {
            // let company_name_key = company_name.clone().unwrap();

            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = self.tokenizer.tokenize(&ticker_symbol);
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
        let company_name_tokens = self.tokenizer.tokenize(&company_name);
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
        let text_doc_tokens = self.tokenizer.tokenize(text);

        if text_doc_tokens.is_empty() {
            println!("No tokens found in the text document. Exiting.");
            return;
        }

        // Get the filtered tokens (tokens present in the TokenMapper)
        let filtered_tokens = self
            .token_mapper
            .get_filtered_tokens(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        // Get the filtered token IDs (IDs present in the TokenMapper)
        let filtered_token_ids = self
            .token_mapper
            .get_filtered_token_ids(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        if filtered_token_ids.is_empty() {
            println!("No token IDs found in the document.");
            return;
        }

        let mut potential_token_id_sequences: HashMap<TickerSymbol, Vec<Vec<usize>>> =
            HashMap::new();

        // Identify token ID sequences which start with the first token of a company token sequence
        for filtered_token_id in &filtered_token_ids {
            if let Some(possible_ticker_symbols) =
                self.company_reverse_token_map.get(filtered_token_id)
            {
                for ticker_symbol in possible_ticker_symbols {
                    if let Some(company_name_variations_token_ids_list) =
                        self.company_token_sequences.get(ticker_symbol)
                    {
                        for company_name_variations_token_ids in
                            company_name_variations_token_ids_list
                        {
                            if company_name_variations_token_ids.is_empty() {
                                continue;
                            }

                            let company_name_first_token_id = company_name_variations_token_ids[0];

                            if *filtered_token_id == company_name_first_token_id {
                                // Add or update the hashmap entry for this ticker_symbol
                                potential_token_id_sequences
                                    .entry(ticker_symbol.clone())
                                    .or_insert_with(Vec::new) // Create an empty Vec if the key doesn't exist
                                    .retain(|existing_vec| {
                                        *existing_vec != *company_name_variations_token_ids
                                    }); // Remove duplicates

                                if !potential_token_id_sequences
                                    .get(&ticker_symbol.clone())
                                    .unwrap()
                                    .contains(&company_name_variations_token_ids)
                                {
                                    potential_token_id_sequences
                                        .get_mut(&ticker_symbol.to_string())
                                        .unwrap()
                                        .push(company_name_variations_token_ids.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut token_pairity_states = Vec::new();

        // Aggregate token pairity states
        for (ticker_symbol, company_token_sequences) in &potential_token_id_sequences {
            for (company_sequence_idx, company_sequence_token_ids) in
                company_token_sequences.iter().enumerate()
            {
                for (filtered_token_idx, filtered_token_id) in filtered_token_ids.iter().enumerate()
                {
                    for (company_sequence_token_idx, company_sequence_token_id) in
                        company_sequence_token_ids.iter().enumerate()
                    {
                        if company_sequence_token_id == filtered_token_id {
                            token_pairity_states.push(TokenParityState {
                                ticker_symbol: ticker_symbol.to_string(),
                                filtered_token_idx,
                                filtered_token_id: *filtered_token_id,
                                company_sequence_idx,
                                company_sequence_token_idx,
                            });
                        }
                    }
                }
            }
        }

        // Reorder token_pairity_states
        token_pairity_states.sort_by(|a, b| {
            (
                a.filtered_token_idx,
                &a.ticker_symbol,
                a.company_sequence_idx,
                a.company_sequence_token_idx,
            )
                .cmp(&(
                    b.filtered_token_idx,
                    &b.ticker_symbol,
                    b.company_sequence_idx,
                    b.company_sequence_token_idx,
                ))
        });

        let mut token_range_states: Vec<TokenRangeState> = Vec::new();
        let mut range_state_map: HashMap<(String, usize), TokenRangeState> = HashMap::new();

        for (ticker_symbol, _company_token_sequences) in &potential_token_id_sequences {
            for token_pairity_state in &token_pairity_states {
                if token_pairity_state.ticker_symbol != *ticker_symbol {
                    continue;
                }

                // Debug print for the token parity state
                println!(
                    "ticker symbol: {}, {}, {}, {:?}",
                    token_pairity_state.ticker_symbol,
                    token_pairity_state.filtered_token_idx,
                    self.token_mapper
                        .get_token_by_id(token_pairity_state.filtered_token_id)
                        .unwrap(),
                    token_pairity_state
                );

                // Aggregate into range states
                let key = (
                    token_pairity_state.ticker_symbol.clone(),
                    token_pairity_state.company_sequence_idx,
                );

                range_state_map
                    .entry(key.clone())
                    .and_modify(|state| {
                        // Ensure contiguity of filtered_token_indices
                        if let Some(&last_idx) = state.filtered_token_indices.last() {
                            if token_pairity_state.filtered_token_idx != last_idx + 1 {
                                // Not contiguous, skip further additions
                                return;
                            }
                        }

                        state
                            .filtered_token_indices
                            .push(token_pairity_state.filtered_token_idx);
                        state
                            .filtered_token_ids
                            .push(token_pairity_state.filtered_token_id);
                        state
                            .company_sequence_token_indices
                            .push(token_pairity_state.company_sequence_token_idx);
                    })
                    .or_insert_with(|| TokenRangeState {
                        ticker_symbol: token_pairity_state.ticker_symbol.clone(),
                        filtered_token_indices: vec![token_pairity_state.filtered_token_idx],
                        filtered_token_ids: vec![token_pairity_state.filtered_token_id],
                        company_sequence_idx: token_pairity_state.company_sequence_idx,
                        company_sequence_length: potential_token_id_sequences
                            .get(&token_pairity_state.ticker_symbol)
                            .unwrap()[token_pairity_state.company_sequence_idx]
                            .len(),
                        company_sequence_token_indices: vec![
                            token_pairity_state.company_sequence_token_idx,
                        ],
                    });
            }
        }

        // Collect range states from the map
        token_range_states.extend(range_state_map.into_values());

        // Print the range states for debugging
        for token_range_state in &token_range_states {
            println!("{:?}", token_range_state);
        }

        // Convert the scores HashMap into a sorted Vec
        // let mut sorted_scores: Vec<(String, f32)> = scores.clone().into_iter().collect();
        // sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sort descending by score

        // TODO: Remove
        println!("Text doc tokens: {:?}", text_doc_tokens);
        println!("Filtered tokens: {:?}", filtered_tokens);
        println!("Filtered token IDs: {:?}", filtered_token_ids);
        println!("Possible matches: {:?}", potential_token_id_sequences);
        // println!("Scores: {:?}", scores);
    }
}
