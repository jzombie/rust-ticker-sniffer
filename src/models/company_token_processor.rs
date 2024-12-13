use std::collections::HashMap;

use crate::types::{
    CompanySymbolList, CompanyTokenSourceType, TokenFrequencyMap, TokenizerVectorToken,
};
use crate::Tokenizer;

type CompanyTokenIndexBySourceType = usize;

type CompanyTokenizedEntry = (
    TokenizerVectorToken,
    CompanyTokenSourceType,
    CompanyTokenIndexBySourceType,
);

/// Index of the company in the `company_symbols_list`
type CompanyIndex = usize;

/// Index of the token within the company's tokens
type CompanyTokenIndex = usize;

type CompanyTokenBinEntry = (CompanyIndex, CompanyTokenIndex);
type CompanyTokenBin = Vec<CompanyTokenBinEntry>;

// For all nested vectors:
//  - Outer vector elements are indexed by company index
//  - Inner vector elements are for multiple entries, per company
pub struct CompanyTokenProcessor<'a> {
    ticker_symbol_tokenizer: &'a Tokenizer,
    text_doc_tokenizer: &'a Tokenizer,
    pub company_symbols_list: &'a CompanySymbolList,
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub tokenized_entries: Vec<Vec<CompanyTokenizedEntry>>,
    pub max_corpus_token_length: usize,
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub token_length_bins: Vec<CompanyTokenBin>,
    pub token_frequency_map: TokenFrequencyMap,
    pub company_name_token_tdidf_scores: HashMap<CompanyIndex, Vec<f32>>,
}

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(
        company_symbols_list: &'a CompanySymbolList,
        ticker_symbol_tokenizer: &'a Tokenizer,
        text_doc_tokenizer: &'a Tokenizer,
    ) -> Self {
        let mut instance = Self {
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            company_symbols_list,
            tokenized_entries: vec![],
            max_corpus_token_length: 0,
            token_length_bins: vec![],
            token_frequency_map: HashMap::new(),
            company_name_token_tdidf_scores: HashMap::new(),
        };

        instance.tokenize_all();
        instance.bin_tokens_by_length();
        instance.compute_company_name_token_tfidf_scores();

        instance
    }

    /// Tokenize and populate tokenized_data and max_corpus_token_length
    fn tokenize_all(&mut self) {
        self.max_corpus_token_length = 0;
        self.tokenized_entries.clear();
        self.token_frequency_map.clear();

        // First pass: Tokenize and determine the maximum token length
        for (symbol, company_name) in self.company_symbols_list.iter() {
            let mut company_tokenized_entries: Vec<CompanyTokenizedEntry> = Vec::new();

            // Handle the symbol token as a single token
            let symbol_token = self
                .ticker_symbol_tokenizer
                .tokenize_to_charcode_vectors(symbol)
                .get(0)
                .cloned(); // Take the first entry, if it exists
            if let Some(symbol_token) = symbol_token {
                company_tokenized_entries.push((symbol_token, CompanyTokenSourceType::Symbol, 0));
                // Token from symbol
            }

            if let Some(company_name) = company_name {
                // Workaround for "urban-gro, Inc."
                // The tokenizer filters on words with uppercase letters, which this does not have
                let uc_company_name = company_name.to_uppercase();

                let company_name_token_vectors = self
                    .text_doc_tokenizer
                    .tokenize_to_charcode_vectors(&uc_company_name);
                for (index_by_source_type, token_vector) in
                    company_name_token_vectors.iter().enumerate()
                {
                    company_tokenized_entries.push((
                        token_vector.to_vec(),
                        CompanyTokenSourceType::CompanyName,
                        index_by_source_type,
                    ));

                    *self
                        .token_frequency_map
                        .entry(token_vector.clone())
                        .or_insert(0) += 1;
                }
            }

            // Store tokenized data for later use
            self.tokenized_entries
                .push(company_tokenized_entries.clone());

            // Update the maximum token length
            for (token, _, _) in &company_tokenized_entries {
                self.max_corpus_token_length = self.max_corpus_token_length.max(token.len());
            }
        }
    }

    fn bin_tokens_by_length(&mut self) {
        self.token_length_bins.clear();

        // Pre-allocate bins after determining the maximum token length
        self.token_length_bins = vec![Vec::new(); self.max_corpus_token_length + 1];

        // Second pass: Populate the bins using stored tokenized data
        for (company_index, company_tokens) in self.tokenized_entries.iter().enumerate() {
            for (tokenized_entry_index, (token, _token_source_type, _token_index_by_source_type)) in
                company_tokens.iter().enumerate()
            {
                let token_length = token.len();
                self.token_length_bins[token_length].push((company_index, tokenized_entry_index));
            }
        }
    }

    fn compute_company_name_token_tfidf_scores(&mut self) {
        self.company_name_token_tdidf_scores.clear();

        let total_documents = self.tokenized_entries.len() as f32;

        for (company_index, company_tokens) in self.tokenized_entries.iter().enumerate() {
            let mut token_tf_map: HashMap<TokenizerVectorToken, f32> = HashMap::new();
            let mut token_count = 0;

            // Calculate TF
            for (token_vector, source_type, _) in company_tokens {
                if *source_type == CompanyTokenSourceType::CompanyName {
                    *token_tf_map.entry(token_vector.clone()).or_insert(0.0) += 1.0;
                    token_count += 1;
                }
            }

            // Create a vector to store TF-IDF scores
            let mut tfidf_vector = Vec::new();

            // Normalize TF and compute TF-IDF
            for (token_vector, tf) in token_tf_map.iter_mut() {
                *tf /= token_count as f32; // Normalize TF
                if let Some(df) = self.token_frequency_map.get(token_vector) {
                    let idf = (total_documents / (1.0 + *df as f32)).ln(); // Compute IDF
                    tfidf_vector.push(*tf * idf); // Store TF-IDF score in vector
                }
            }

            // Store the vector in the map
            self.company_name_token_tdidf_scores
                .insert(company_index, tfidf_vector);
        }
    }

    pub fn get_company_name_token_vectors(
        &self,
        company_index: usize,
    ) -> Option<Vec<TokenizerVectorToken>> {
        // Retrieve the tokenized entries for the given company index
        let tokenized_entries = self.tokenized_entries.get(company_index)?;

        // Filter tokens that are of the `CompanyName` source type and map them to strings
        let company_name_tokens_vectors: Vec<TokenizerVectorToken> = tokenized_entries
            .iter()
            .filter_map(|(token_vector, token_source_type, _)| {
                if *token_source_type == CompanyTokenSourceType::CompanyName {
                    // Convert the token to a string (adjust based on actual token structure)
                    Some(token_vector.clone())
                } else {
                    None
                }
            })
            .collect();

        // Return None if no tokens are found, otherwise return the tokens
        if company_name_tokens_vectors.is_empty() {
            None
        } else {
            Some(company_name_tokens_vectors)
        }
    }

    pub fn get_company_name_tokens(&self, company_index: usize) -> Option<Vec<String>> {
        // Retrieve the tokenized entries for the given company index
        let tokenized_entries = self.tokenized_entries.get(company_index)?;

        // Filter tokens that are of the `CompanyName` source type and map them to strings
        let company_name_tokens: Vec<String> = tokenized_entries
            .iter()
            .filter_map(|(token_vector, token_source_type, _)| {
                if *token_source_type == CompanyTokenSourceType::CompanyName {
                    // Convert the token to a string (adjust based on actual token structure)
                    Some(Tokenizer::charcode_vector_to_token(token_vector))
                } else {
                    None
                }
            })
            .collect();

        // Return None if no tokens are found, otherwise return the tokens
        if company_name_tokens.is_empty() {
            None
        } else {
            Some(company_name_tokens)
        }
    }

    pub fn get_total_company_name_tokens(&self, company_index: usize) -> usize {
        match self.get_company_name_tokens(company_index) {
            Some(company_name_tokens) => company_name_tokens.len(),
            None => 0,
        }
    }

    // TODO: Rename
    /// Calculates the total length of all tokens in a company's name.
    pub fn calculate_summed_company_token_length(&self, company_index: usize) -> usize {
        match self.get_company_name_tokens(company_index) {
            Some(company_name_tokens) => {
                let mut total_token_length = 0;
                for token in company_name_tokens {
                    total_token_length += token.len()
                }
                total_token_length
            }
            None => 0,
        }
    }
}
