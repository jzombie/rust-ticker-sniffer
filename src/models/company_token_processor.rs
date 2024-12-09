use crate::types::{CompanySymbolsList, CompanyTokenSourceType};
use crate::utils::tokenize;

// TODO: Use numeric data
type TokenizedEntry = Vec<(String, CompanyTokenSourceType)>;

/// Index of the company in the `company_symbols_list`
type CompanyIndex = usize;

/// Index of the token within the company's tokens
type TokenIndex = usize;

/// A bin of tokens, where each token is represented by its company index and token index.
type TokenBin = Vec<(CompanyIndex, TokenIndex)>;

pub struct CompanyTokenProcessor<'a> {
    pub company_symbols_list: &'a CompanySymbolsList,
    pub tokenized_entries: Vec<TokenizedEntry>,
    pub max_corpus_token_length: usize,
    pub token_length_bins: Vec<TokenBin>,
}

pub struct CompanyTokenQueryResult<'a> {
    pub company_index: CompanyIndex,
    pub token_index: TokenIndex,
    pub token: &'a str,
    pub source_type: CompanyTokenSourceType,
    pub symbol: &'a str,
    pub company_name: Option<&'a str>,
    pub company_tokens: &'a [(String, CompanyTokenSourceType)],
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
            let mut company_tokens: Vec<(String, CompanyTokenSourceType)> = Vec::new();

            // Handle the symbol token as a single token
            let symbol_token = tokenize(symbol).get(0).cloned(); // Take the first entry, if it exists
            if let Some(symbol_token) = symbol_token {
                company_tokens.push((symbol_token, CompanyTokenSourceType::Symbol));
                // Token from symbol
            }

            if let Some(name) = company_name {
                let name_tokens = tokenize(name);
                for token in name_tokens {
                    company_tokens.push((token, CompanyTokenSourceType::CompanyName));
                    // Token from company name
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

    /// Note: An Iterator is used on the output to prevent eager execution,
    /// letting the consumer determine if it should proceed to the next result.
    ///
    /// Note: `token_end_index` is non-inclusive
    pub fn search_token_space(
        &'a self,
        min_token_length: usize,
        max_token_length: usize,
        token_start_index: usize,
        token_end_index: usize,
        include_source_types: &'a [CompanyTokenSourceType],
    ) -> impl Iterator<Item = CompanyTokenQueryResult<'a>> + 'a {
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
                        let (token, source_type) =
                            &self.tokenized_entries[company_index][token_index];
                        let (symbol, company_name) = &self.company_symbols_list[company_index];
                        let company_tokens = &self.tokenized_entries[company_index];

                        CompanyTokenQueryResult {
                            company_index,
                            token_index,
                            token,
                            source_type: *source_type,
                            symbol,
                            company_name: company_name.as_deref(),
                            company_tokens,
                        }
                    })
            })
    }

    // pub fn query_tokens_by_length(&self, length: usize) -> Option<&TokenBin> {
    //     self.token_length_bins.get(length)
    // }
}
