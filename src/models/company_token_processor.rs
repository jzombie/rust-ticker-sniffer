use crate::types::{CompanySymbolsList, CompanyTokenSourceType, TokenizerVectorTokenType};
use crate::Tokenizer;

type CompanyTokenIndexBySourceType = usize;

type CompanyTokenizedEntry = (
    TokenizerVectorTokenType,
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
//  - Outer vector elements are by company index
//  - Inner vector elements are for multiple entries, per company
pub struct CompanyTokenProcessor<'a> {
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    pub company_symbols_list: &'a CompanySymbolsList,
    pub company_name_lengths: Vec<usize>,
    pub company_name_token_counts: Vec<usize>,
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub tokenized_entries: Vec<Vec<CompanyTokenizedEntry>>,
    pub max_corpus_token_length: usize,
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub token_length_bins: Vec<CompanyTokenBin>,
}

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(company_symbols_list: &'a CompanySymbolsList) -> Self {
        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser();

        let mut instance = Self {
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            company_symbols_list,
            company_name_lengths: vec![],
            company_name_token_counts: vec![],
            tokenized_entries: vec![],
            max_corpus_token_length: 0,
            token_length_bins: vec![],
        };

        instance.tokenize_all();
        instance.bin_tokens_by_length();

        instance
    }

    /// Tokenize and populate tokenized_data and max_corpus_token_length
    fn tokenize_all(&mut self) {
        self.company_name_lengths.clear();
        self.company_name_token_counts.clear();
        self.max_corpus_token_length = 0;
        self.tokenized_entries.clear();

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
                self.company_name_lengths.push(company_name.len());

                let mut company_name_token_count: usize = 0;

                let company_name_token_vectors = self
                    .text_doc_tokenizer
                    .tokenize_to_charcode_vectors(company_name);
                for (index_by_source_type, token) in company_name_token_vectors.iter().enumerate() {
                    company_tokenized_entries.push((
                        token.to_vec(),
                        CompanyTokenSourceType::CompanyName,
                        index_by_source_type,
                    ));

                    company_name_token_count += 1;
                    // Token from company name
                }

                self.company_name_token_counts
                    .push(company_name_token_count);
            } else {
                self.company_name_lengths.push(0);
                self.company_name_token_counts.push(0);
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

    pub fn get_company_name_tokens(&self, company_index: usize) -> Option<Vec<String>> {
        // Retrieve the tokenized entries for the given company index
        let tokenized_entries = self.tokenized_entries.get(company_index)?;

        // Filter tokens that are of the `CompanyName` source type and map them to strings
        let company_name_tokens: Vec<String> = tokenized_entries
            .iter()
            .filter_map(|(token, token_source_type, _)| {
                if *token_source_type == CompanyTokenSourceType::CompanyName {
                    // Convert the token to a string (adjust based on actual token structure)
                    Some(self.text_doc_tokenizer.charcode_vector_to_token(token))
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
}
