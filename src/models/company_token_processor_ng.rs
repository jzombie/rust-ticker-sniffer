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

        for (symbol, token_id_vectors) in &self.company_name_token_map {
            for company_token_ids in token_id_vectors {
                // Inline logic for contiguous match
                let mut company_idx = 0;
                let mut filtered_idx = 0;

                while company_idx < company_token_ids.len()
                    && filtered_idx < filtered_token_ids.len()
                {
                    if company_token_ids[company_idx] == filtered_token_ids[filtered_idx] {
                        company_idx += 1;
                    }
                    filtered_idx += 1;

                    // If we've matched all company_token_ids in order, it's a contiguous match
                    if company_idx == company_token_ids.len() {
                        *match_counts.entry(symbol.clone()).or_insert(0) += 1;
                        break; // No need to check further for this company_token_ids
                    }
                }
            }
        }

        // Convert matches to a Vec and sort by relevance
        let mut possible_matches: Vec<(String, usize)> = match_counts.into_iter().collect();
        possible_matches.sort_by(|a, b| b.1.cmp(&a.1));

        // Print all relevant details, including filtered tokens
        println!("Text doc tokens: {:?}", text_doc_tokens); // Original tokens
        println!("Filtered tokens: {:?}", filtered_tokens); // Tokens present in the TokenMapper
        println!("Filtered token IDs: {:?}", filtered_token_ids); // Corresponding token IDs
        println!("Possible matches: {:?}", possible_matches); // Matches with counts
    }
}
