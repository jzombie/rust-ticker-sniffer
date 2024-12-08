use crate::types::CompanySymbolsList;
use crate::utils::tokenize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)] // Use a numeric representation for efficiency
pub enum TokenSourceType {
    Symbol = 0,      // Tokens derived from the symbol
    CompanyName = 1, // Tokens derived from the company name
}

// TODO: Use numeric data
type TokenizedEntry = Vec<(String, TokenSourceType)>;

/// Index of the company in the `company_symbols_list`
type CompanyIndex = usize;

/// Index of the token within the company's tokens
type TokenIndex = usize;

/// A bin of tokens, where each token is represented by its company index and token index.
type TokenBin = Vec<(CompanyIndex, TokenIndex)>;

pub struct TokenProcessor<'a> {
    pub company_symbols_list: &'a CompanySymbolsList,
    pub tokenized_entries: Vec<TokenizedEntry>,
    pub max_corpus_token_length: usize,
    pub token_length_bins: Vec<TokenBin>,
}

impl<'a> TokenProcessor<'a> {
    pub fn new(company_symbols_list: &'a CompanySymbolsList) -> Self {
        let mut instance = Self {
            company_symbols_list,
            tokenized_entries: Vec::new(),
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
        self.tokenized_entries.clear();

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
            self.tokenized_entries.push(company_tokens.clone());

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
        for (company_index, company_tokens) in self.tokenized_entries.iter().enumerate() {
            for (token_index, (token, _)) in company_tokens.iter().enumerate() {
                let token_length = token.len();
                self.token_length_bins[token_length].push((company_index, token_index));
            }
        }
    }

    pub fn query_tokens_by_length(&self, length: usize) -> Option<&TokenBin> {
        self.token_length_bins.get(length)
    }
}