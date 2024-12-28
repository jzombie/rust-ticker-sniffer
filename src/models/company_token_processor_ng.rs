use crate::types::CompanySymbolList;
use crate::TokenMapper;
use crate::Tokenizer;
use std::collections::{HashMap, HashSet};

// TODO: Rename without the `Ng` suffix
pub struct CompanyTokenProcessorNg<'a> {
    company_symbol_list: &'a CompanySymbolList,
    token_mapper: TokenMapper,
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    // TODO: Use id instead of String
    company_name_token_map: HashMap<String, Vec<Vec<usize>>>,
    reverse_token_map: HashMap<usize, Vec<String>>,
}

impl<'a> CompanyTokenProcessorNg<'a> {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        let mut instance = CompanyTokenProcessorNg {
            company_symbol_list,
            token_mapper: TokenMapper::new(),
            ticker_symbol_tokenizer: Tokenizer::ticker_symbol_parser(),
            text_doc_tokenizer: Tokenizer::text_doc_parser(),
            company_name_token_map: HashMap::with_capacity(company_symbol_list.len()),
            reverse_token_map: HashMap::new(),
        };

        instance.ingest_company_tokens();

        instance
    }

    /// Ingests tokens from the company symbol list
    fn ingest_company_tokens(&mut self) {
        self.company_name_token_map.clear();
        self.reverse_token_map.clear();

        for (ticker_symbol, company_name, alt_company_names) in self.company_symbol_list {
            // let company_name_key = company_name.clone().unwrap();

            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                let ticker_symbol_token_id = self.token_mapper.upsert_token(&ticker_symbol_token);
                all_company_name_token_ids.push(vec![ticker_symbol_token_id]);

                // Populate reverse map
                self.reverse_token_map
                    .entry(ticker_symbol_token_id)
                    .or_insert_with(Vec::new)
                    .push(ticker_symbol.clone());
            }

            if let Some(company_name) = company_name {
                let company_name_token_ids = self.process_company_name_tokens(&company_name);
                all_company_name_token_ids.push(company_name_token_ids.clone());

                // Populate reverse map
                for token_id in company_name_token_ids {
                    self.reverse_token_map
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
                    self.reverse_token_map
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol.clone());
                }
            }

            // Insert the collected token IDs into the map
            self.company_name_token_map
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

    pub fn process_text_doc(&mut self, text: &str) {
        // Tokenize the input text
        let text_doc_tokens = self.text_doc_tokenizer.tokenize(text);

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

        // Collect matches based on token IDs
        let mut match_counts: HashMap<String, usize> = HashMap::new();

        for token_id in &filtered_token_ids {
            if let Some(companies) = self.reverse_token_map.get(token_id) {
                for company in companies {
                    if let Some(token_id_vectors) = self.company_name_token_map.get(company) {
                        let mut max_match_count = 0;

                        for token_vector in token_id_vectors {
                            if token_vector.is_empty() {
                                continue;
                            }

                            let mut company_idx = 0;
                            let mut filtered_idx = 0;
                            let mut current_match_count = 0;

                            // Check for consecutive matches
                            while company_idx < token_vector.len()
                                && filtered_idx < filtered_token_ids.len()
                            {
                                if token_vector[company_idx] == filtered_token_ids[filtered_idx] {
                                    company_idx += 1;
                                    current_match_count += 1;
                                } else {
                                    // Reset match count if not consecutive
                                    current_match_count = 0;
                                    company_idx = 0; // Restart match from the beginning of the token vector
                                }

                                filtered_idx += 1;

                                // If all tokens match consecutively, stop checking further
                                if company_idx == token_vector.len() {
                                    break;
                                }
                            }

                            // Update max match count for this company
                            max_match_count = max_match_count.max(current_match_count);
                        }

                        if max_match_count > 0 {
                            match_counts
                                .entry(company.clone())
                                .and_modify(|count| *count = (*count).max(max_match_count))
                                .or_insert(max_match_count);
                        }
                    }
                }
            }
        }

        // Convert matches to a Vec and sort by relevance
        let mut possible_matches: Vec<(String, usize)> = match_counts.into_iter().collect();
        possible_matches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // Sort by count, then alphabetically

        // Print all relevant details
        println!("Text doc tokens: {:?}", text_doc_tokens);
        println!("Filtered tokens: {:?}", filtered_tokens);
        println!("Filtered token IDs: {:?}", filtered_token_ids);
        println!("Possible matches: {:?}", possible_matches);
    }
}
