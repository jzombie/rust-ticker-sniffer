use crate::types::{CompanySymbolsList, CompanyTokenSourceType};
use crate::utils::tokenize_to_charcode_vectors;

type CompanyVectorTokenType = Vec<u32>;

type CompanyTokenizedEntry = (CompanyVectorTokenType, CompanyTokenSourceType);

/// Index of the company in the `company_symbols_list`
type CompanyIndex = usize;

/// Index of the token within the company's tokens
type CompanyTokenIndex = usize;

type CompanyTokenBinEntry = (CompanyIndex, CompanyTokenIndex);

pub struct CompanyTokenProcessor<'a> {
    pub company_symbols_list: &'a CompanySymbolsList,
    pub tokenized_entries: Vec<Vec<CompanyTokenizedEntry>>,
    pub max_corpus_token_length: usize,
    pub token_length_bins: Vec<Vec<CompanyTokenBinEntry>>,
}

pub struct CompanyFilteredTokenResult<'a> {
    pub company_index: CompanyIndex,
    pub token_index: CompanyTokenIndex,
    pub token_vector: &'a CompanyVectorTokenType,
    pub source_type: CompanyTokenSourceType,
    pub symbol: &'a str,
    pub company_name: Option<&'a str>,
    pub company_tokenized_entries: &'a Vec<CompanyTokenizedEntry>,
}
impl<'a> CompanyTokenProcessor<'a> {
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
            let mut company_tokenized_entries: Vec<CompanyTokenizedEntry> = Vec::new();

            // Handle the symbol token as a single token
            let symbol_token = tokenize_to_charcode_vectors(symbol).get(0).cloned(); // Take the first entry, if it exists
            if let Some(symbol_token) = symbol_token {
                company_tokenized_entries.push((symbol_token, CompanyTokenSourceType::Symbol));
                // Token from symbol
            }

            if let Some(name) = company_name {
                let name_tokens = tokenize_to_charcode_vectors(name);
                for token in name_tokens {
                    company_tokenized_entries.push((token, CompanyTokenSourceType::CompanyName));
                    // Token from company name
                }
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

    /// Note: An Iterator is used on the output for lazy execution, letting the
    /// consumer decide whether or not to proceed to the next result.
    ///
    /// Note: `token_end_index` is non-inclusive
    pub fn filter_token_space(
        &'a self,
        min_token_length: usize,
        max_token_length: usize,
        token_start_index: usize,
        token_end_index: usize,
        include_source_types: &'a [CompanyTokenSourceType],
    ) -> impl Iterator<Item = CompanyFilteredTokenResult<'a>> + 'a {
        // Iterate over the range of token lengths
        (min_token_length..=max_token_length)
            .filter_map(move |length| self.token_length_bins.get(length)) // Only process bins that exist
            .flat_map(move |bin| {
                bin.iter()
                    // Filter tokens by their index within the specified range
                    .filter(move |&&(_, token_index)| {
                        token_index >= token_start_index && token_index < token_end_index
                    })
                    .filter(move |&&(company_index, token_index)| {
                        let (_, source_type) = &self.tokenized_entries[company_index][token_index];
                        // Include only tokens with source types in the provided list
                        include_source_types.contains(source_type)
                    })
                    .map(move |&(company_index, token_index)| {
                        let (token_vector, source_type) =
                            &self.tokenized_entries[company_index][token_index];
                        let (symbol, company_name) = &self.company_symbols_list[company_index];
                        let company_tokenized_entries = &self.tokenized_entries[company_index];

                        CompanyFilteredTokenResult {
                            company_index,
                            token_index,
                            token_vector,
                            source_type: *source_type,
                            symbol,
                            company_name: company_name.as_deref(),
                            company_tokenized_entries,
                        }
                    })
            })
    }

    // pub fn query_tokens_by_length(&self, length: usize) -> Option<&TokenBin> {
    //     self.token_length_bins.get(length)
    // }
}
