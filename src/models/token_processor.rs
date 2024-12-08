use crate::types::{CompanySymbolsList, TokenSourceType};

pub struct TokenProcessor {
    company_symbols_list: CompanySymbolsList,
    tokenized_data: Vec<Vec<(String, TokenSourceType)>>,
    max_corpus_token_length: usize,
    token_length_bins: Vec<Vec<(usize, usize)>>,
}

impl TokenProcessor {
    pub fn new(company_symbols_list: CompanySymbolsList) -> Self {
        Self {
            company_symbols_list,
            tokenized_data: Vec::new(),
            max_corpus_token_length: 0,
            token_length_bins: Vec::new(),
        }
    }

    pub fn tokenize_all(&mut self) {
        // Tokenize and populate tokenized_data and max_corpus_token_length
    }

    pub fn bin_tokens_by_length(&mut self) {
        // Populate token_length_bins based on tokenized_data
    }

    // pub fn query_tokens_by_length(&self, length: usize) -> Option<&Vec<(usize, usize)>> {
    //     // Return tokens of a given length
    // }
}
