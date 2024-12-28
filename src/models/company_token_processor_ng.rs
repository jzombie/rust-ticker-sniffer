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

        // Get the filtered tokens (tokens present in the TokenMapper)
        let filtered_tokens = self
            .token_mapper
            .get_filtered_tokens(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        // Get the filtered token IDs (IDs present in the TokenMapper)
        let filtered_token_ids = self
            .token_mapper
            .get_filtered_token_ids(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        // Collect all companies associated with the token IDs and track matches
        let mut match_counts: HashMap<String, HashSet<usize>> = HashMap::new();
        for token_id in &filtered_token_ids {
            if let Some(companies) = self.reverse_token_map.get(token_id) {
                for company in companies {
                    match_counts
                        .entry(company.clone())
                        .or_insert_with(HashSet::new)
                        .insert(*token_id);
                }
            }
        }

        // Flatten the HashMap into a Vec with the count of unique matches
        let mut possible_matches: Vec<(String, usize)> = match_counts
            .into_iter()
            .map(|(company, token_ids)| (company, token_ids.len()))
            .collect();

        // Sort by hit count (descending)
        possible_matches.sort_by(|a, b| b.1.cmp(&a.1));

        // Print all relevant details
        println!("Text doc tokens: {:?}", text_doc_tokens); // Original
        println!("Filtered tokens: {:?}", filtered_tokens); // Original
        println!("Filtered token IDs: {:?}", filtered_token_ids); // Original
        println!("Possible matches: {:?}", possible_matches); // Corrected hit counts
    }

    // pub fn process_company_name(&mut self, company_name: &str) -> Vec<usize> {
    //     let tokens = self.tokenizer.tokenize(company_name);
    //     tokens
    //         .iter()
    //         .map(|token| self.token_mapper.upsert_token(token))
    //         .collect()
    // }

    // pub fn get_token_id(&self, token: &str) -> Option<usize> {
    //     self.token_mapper.get_token_id(token)
    // }

    // pub fn get_token_count(&self) -> usize {
    //     self.token_mapper.token_count()
    // }
}
