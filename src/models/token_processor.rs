use crate::types::{CompanySymbolsList, TokenSourceType};
use crate::utils::tokenize;

pub struct TokenProcessor<'a> {
    pub company_symbols_list: &'a CompanySymbolsList,
    pub tokenized_data: Vec<Vec<(String, TokenSourceType)>>,
    pub max_corpus_token_length: usize,
    pub token_length_bins: Vec<Vec<(usize, usize)>>,
}

impl<'a> TokenProcessor<'a> {
    pub fn new(company_symbols_list: &'a CompanySymbolsList) -> Self {
        let mut instance = Self {
            company_symbols_list,
            tokenized_data: Vec::new(),
            max_corpus_token_length: 0,
            token_length_bins: Vec::new(),
        };

        instance.tokenize_all();
        instance.bin_tokens_by_length();

        instance
    }

    /// Tokenize and populate tokenized_data and max_corpus_token_length
    fn tokenize_all(&mut self) {
        self.max_corpus_token_length = 0;
        self.tokenized_data.clear();

        // First pass: Tokenize and determine the maximum token length
        for (symbol, company_name) in self.company_symbols_list.iter() {
            let mut company_tokens: Vec<(String, TokenSourceType)> = Vec::new();

            // Handle the symbol token as a single token
            let symbol_token = tokenize(symbol).get(0).cloned(); // Take the first entry, if it exists
            if let Some(symbol_token) = symbol_token {
                company_tokens.push((symbol_token, TokenSourceType::Symbol)); // Token from symbol
            }

            if let Some(name) = company_name {
                let name_tokens = tokenize(name);
                for token in name_tokens {
                    company_tokens.push((token, TokenSourceType::CompanyName)); // Token from company name
                }
            }

            // Store tokenized data for later use
            self.tokenized_data.push(company_tokens.clone());

            // Update the maximum token length
            for (token, _) in &company_tokens {
                self.max_corpus_token_length = self.max_corpus_token_length.max(token.len());
            }
        }
    }

    fn bin_tokens_by_length(&mut self) {
        self.token_length_bins.clear();

        // TODO: Store numeric tokens instead
        // Outer vector element is by company index
        // Inner vector element is individual tokens for respective company

        // Pre-allocate bins after determining the maximum token length
        self.token_length_bins = vec![Vec::new(); self.max_corpus_token_length + 1];

        // Second pass: Populate the bins using stored tokenized data
        for (company_index, company_tokens) in self.tokenized_data.iter().enumerate() {
            for (token_index, (token, _)) in company_tokens.iter().enumerate() {
                let token_length = token.len();
                self.token_length_bins[token_length].push((company_index, token_index));
            }
        }
    }

    // pub fn query_tokens_by_length(&self, length: usize) -> Option<&Vec<(usize, usize)>> {
    //     // Return tokens of a given length
    // }
}
