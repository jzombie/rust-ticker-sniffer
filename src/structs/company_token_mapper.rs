use std::collections::HashMap;

use crate::types::{
    CompanySequenceIndex, CompanySymbolList, TickerSymbol, TickerSymbolTokenId, TokenId,
};

use crate::{Error, TokenMapper, Tokenizer};

pub struct CompanyTokenMapper {
    pub token_mapper: TokenMapper,
    pub ticker_symbol_tokenizer: Tokenizer,
    pub text_doc_tokenizer: Tokenizer,
    pub ticker_symbol_map: HashMap<TickerSymbol, TickerSymbolTokenId>,
    pub reverse_ticker_symbol_map: HashMap<TokenId, TickerSymbol>,
    pub company_token_sequences_map: HashMap<TickerSymbolTokenId, Vec<Vec<TokenId>>>,
    pub company_reverse_token_map: HashMap<TokenId, Vec<TickerSymbolTokenId>>,
}

impl CompanyTokenMapper {
    /// Creates a new instance of `CompanyTokenMapper` by processing the provided company symbol list.
    ///
    /// # Arguments
    /// * `company_symbol_list` - A reference to the list of company symbols.
    /// * `case_sensitive` - Whether or not the text document should be filtered by case sensitivity.
    ///
    /// # Errors
    /// Returns an error if token ingestion fails.
    pub fn new(
        company_symbol_list: &CompanySymbolList,
        case_sensitive: bool,
    ) -> Result<Self, Error> {
        let token_mapper = TokenMapper::new();

        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser(case_sensitive);

        let mut instance = CompanyTokenMapper {
            token_mapper,
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            reverse_ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            company_token_sequences_map: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        };

        instance.ingest_company_tokens(company_symbol_list)?;

        Ok(instance)
    }

    /// Clears all token maps and associated data from the mapper.
    fn clear(&mut self) {
        self.company_token_sequences_map.clear();
        self.company_reverse_token_map.clear();
        self.ticker_symbol_map.clear();
        self.reverse_ticker_symbol_map.clear();
    }

    /// Ingests tokens from the provided company symbol list into the mapper.
    ///
    /// # Arguments
    /// * `company_symbol_list` - A reference to the list of company symbols.
    ///
    /// # Errors
    /// Returns an error if the ingestion process fails.
    fn ingest_company_tokens(
        &mut self,
        company_symbol_list: &CompanySymbolList,
    ) -> Result<(), Error> {
        self.clear();

        for (ticker_symbol, company_name, alt_company_names) in company_symbol_list {
            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                let ticker_symbol_token_id = self.token_mapper.upsert_token(&ticker_symbol_token);

                self.ticker_symbol_map
                    .insert(ticker_symbol.clone(), ticker_symbol_token_id);

                self.reverse_ticker_symbol_map
                    .insert(ticker_symbol_token_id, ticker_symbol.clone());
            }

            let ticker_symbol_token_id = *self.get_ticker_symbol_token_id(ticker_symbol)?;

            if let Some(company_name) = company_name {
                let company_name_token_ids = self.process_company_name_tokens(company_name);
                all_company_name_token_ids.push(company_name_token_ids.clone());

                // Populate reverse map
                for token_id in company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id)
                        .or_default()
                        .push(ticker_symbol_token_id);
                }
            }

            // Process alternate company names
            for alt_company_name in alt_company_names {
                let alt_company_name_token_ids = self.process_company_name_tokens(alt_company_name);
                all_company_name_token_ids.push(alt_company_name_token_ids.clone());

                // Populate reverse map
                for token_id in alt_company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id)
                        .or_default()
                        .push(ticker_symbol_token_id);
                }
            }

            // Insert the collected token IDs into the map
            self.company_token_sequences_map
                .entry(ticker_symbol_token_id)
                .or_default()
                .extend(all_company_name_token_ids);
        }

        Ok(())
    }

    /// Tokenizes the given company name and processes its tokens into unique token IDs.
    ///
    /// # Arguments
    /// * `company_name` - A reference to the company name as a string.
    ///
    /// # Returns
    /// A vector of token IDs for the processed company name.
    fn process_company_name_tokens(&mut self, company_name: &str) -> Vec<TokenId> {
        // Note: Company name is explcitly converted to upper-case here as this is
        // part of the token ingestion and the data source isn't guaranteed to be 100% normalized
        // with uppercase words
        let uppercased_name = company_name.to_uppercase();

        let company_name_tokens = self.text_doc_tokenizer.tokenize(&uppercased_name);
        let mut company_name_token_ids = Vec::new();
        for token in company_name_tokens {
            let token_id = self.token_mapper.upsert_token(&token);
            company_name_token_ids.push(token_id);
        }

        company_name_token_ids
    }

    /// Retrieves the ticker symbol corresponding to the given token ID.
    ///
    /// # Arguments
    /// * `token_id` - The ID of the token to look up.
    ///
    /// # Errors
    /// Returns an error if no ticker symbol is associated with the provided token ID.
    pub fn get_ticker_symbol_by_token_id(
        &self,
        token_id: &TokenId,
    ) -> Result<&TickerSymbol, Error> {
        match self.reverse_ticker_symbol_map.get(token_id) {
            Some(ticker_symbol) => Ok(ticker_symbol),
            None => Err(Error::ParserError(
                format!("Could not ticker from token token id: {}", token_id).to_string(),
            )),
        }
    }

    /// Retrieves the token ID corresponding to the given ticker symbol.
    ///
    /// # Arguments
    /// * `ticker_symbol` - A reference to the ticker symbol as a string.
    ///
    /// # Errors
    /// Returns an error if the token ID is not found.
    pub fn get_ticker_symbol_token_id(
        &self,
        ticker_symbol: &TickerSymbol,
    ) -> Result<&TokenId, Error> {
        match self.ticker_symbol_map.get(ticker_symbol) {
            Some(token_id) => Ok(token_id),
            None => Err(Error::ParserError("Could not obtain token id".to_string())),
        }
    }

    /// Retrieves the maximum sequence length for a company's token sequence.
    ///
    /// # Arguments
    /// * `ticker_symbol_token_id` - The token ID of the ticker symbol.
    /// * `company_sequence_idx` - The index of the company sequence.
    ///
    /// # Returns
    /// The maximum length of the token sequence, if found.
    pub fn get_company_token_sequence_max_length(
        &self,
        ticker_symbol_token_id: &TickerSymbolTokenId,
        company_sequence_idx: CompanySequenceIndex,
    ) -> Option<usize> {
        self.company_token_sequences_map
            .get(ticker_symbol_token_id)
            .and_then(|seq| seq.get(company_sequence_idx).map(|s| s.len()))
    }
}
