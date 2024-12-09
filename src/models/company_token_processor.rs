use crate::types::{CompanySymbolsList, CompanyTokenSourceType, TokenizerVectorTokenType};
use crate::Tokenizer;

type CompanyTokenizedEntry = (TokenizerVectorTokenType, CompanyTokenSourceType);

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
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub tokenized_entries: Vec<Vec<CompanyTokenizedEntry>>,
    pub max_corpus_token_length: usize,
    // TODO: Using a flat buffer would be more performant, but something would
    // need to handle the offsets accordingly
    pub token_length_bins: Vec<CompanyTokenBin>,
}

// TODO: Remove?
// pub struct CompanyFilteredTokenResult<'a> {
//     pub company_index: CompanyIndex,
//     pub token_index: CompanyTokenIndex,
//     pub token_vector: &'a CompanyVectorTokenType,
//     pub source_type: CompanyTokenSourceType,
//     pub symbol: &'a str,
//     pub company_name: Option<&'a str>,
//     pub company_tokenized_entries: &'a Vec<CompanyTokenizedEntry>,
// }

impl<'a> CompanyTokenProcessor<'a> {
    pub fn new(company_symbols_list: &'a CompanySymbolsList) -> Self {
        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser();

        let mut instance = Self {
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            company_symbols_list,
            company_name_lengths: vec![],
            tokenized_entries: vec![],
            max_corpus_token_length: 0,
            token_length_bins: vec![],
        };

        instance.tokenize_all();
        instance.bin_tokens_by_length();

        instance
    }

    // pub fn get_company_name_token_vector_at_index(
    //     &self,
    //     company_index: usize,
    //     company_token_index: usize,
    // ) -> Option<&TokenizerVectorTokenType> {
    //     // Retrieve tokenized entries for the specified company index
    //     let company_tokenized_entries = self.tokenized_entries.get(company_index)?;

    //     // Find the specific token entry matching the given token index and source type
    //     company_tokenized_entries
    //         .iter()
    //         .filter(|entry| entry.1 == CompanyTokenSourceType::CompanyName)
    //         .nth(company_token_index)
    //         .map(|entry| &entry.0)
    // }

    /// Tokenize and populate tokenized_data and max_corpus_token_length
    fn tokenize_all(&mut self) {
        self.company_name_lengths.clear();
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
                company_tokenized_entries.push((symbol_token, CompanyTokenSourceType::Symbol));
                // Token from symbol
            }

            if let Some(company_name) = company_name {
                self.company_name_lengths.push(company_name.len());

                let company_name_token_vectors = self
                    .text_doc_tokenizer
                    .tokenize_to_charcode_vectors(company_name);
                for token in company_name_token_vectors {
                    company_tokenized_entries.push((token, CompanyTokenSourceType::CompanyName));
                    // Token from company name
                }
            } else {
                self.company_name_lengths.push(0);
            }

            // Store tokenized data for later use
            self.tokenized_entries
                .push(company_tokenized_entries.clone());

            // Update the maximum token length
            for (token, _) in &company_tokenized_entries {
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
            for (token_index, (token, _)) in company_tokens.iter().enumerate() {
                let token_length = token.len();
                self.token_length_bins[token_length].push((company_index, token_index));
            }
        }
    }

    // TODO: Remove?
    //
    // pub fn get_token_len_bins(
    //     &self,
    //     min_token_length: usize,
    //     max_token_length: usize,
    // ) -> Vec<&CompanyTokenBinEntry> {
    //     self.token_length_bins
    //         .iter()
    //         .enumerate()
    //         .filter(|(length, bin)| {
    //             *length >= min_token_length && *length <= max_token_length && !bin.is_empty()
    //         })
    //         .flat_map(|(_, bin)| bin.iter()) // Flatten all bins into a single iterator of references
    //         .collect()
    // }

    // TODO: Maybe useful for debugging, but awful for performance
    //
    // Note: An Iterator is used on the output for lazy execution, letting the
    // consumer decide whether or not to proceed to the next result.
    //
    // Note: `token_end_index` is non-inclusive
    // pub fn filter_token_space(
    //     &'a self,
    //     min_token_length: usize,
    //     max_token_length: usize,
    //     token_start_index: usize,
    //     token_end_index: usize,
    //     include_source_types: &'a [CompanyTokenSourceType],
    // ) -> impl Iterator<Item = CompanyFilteredTokenResult<'a>> + 'a {
    //     // Pre-filter token lengths with non-empty bins
    //     self.token_length_bins
    //         .iter()
    //         .enumerate()
    //         .filter(move |(length, bin)| {
    //             *length >= min_token_length && *length <= max_token_length && !bin.is_empty()
    //         })
    //         .flat_map(move |(_, bin)| {
    //             bin.iter()
    //                 // Combine filters for better performance
    //                 .filter(move |&&(company_index, token_index)| {
    //                     token_index >= token_start_index
    //                         && token_index < token_end_index
    //                         && include_source_types
    //                             .contains(&self.tokenized_entries[company_index][token_index].1)
    //                 })
    //                 .map(move |&(company_index, token_index)| {
    //                     // Minimize repeated lookups
    //                     let (token_vector, source_type) =
    //                         &self.tokenized_entries[company_index][token_index];
    //                     let (symbol, company_name) = &self.company_symbols_list[company_index];
    //                     let company_tokenized_entries = &self.tokenized_entries[company_index];

    //                     CompanyFilteredTokenResult {
    //                         company_index,
    //                         token_index,
    //                         token_vector,
    //                         source_type: *source_type,
    //                         symbol,
    //                         company_name: company_name.as_deref(),
    //                         company_tokenized_entries,
    //                     }
    //                 })
    //         })
    // }

    // pub fn query_tokens_by_length(&self, length: usize) -> Option<&TokenBin> {
    //     self.token_length_bins.get(length)
    // }
}
