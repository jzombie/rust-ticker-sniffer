use std::collections::HashMap;

use crate::types::{
    CompanySequenceIndex, CompanySymbolList, TickerSymbol, TickerSymbolTokenId, TokenId,
    TokenVector,
};

use crate::{TokenMapper, Tokenizer};

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
    pub fn new(company_symbol_list: &CompanySymbolList) -> Self {
        let token_mapper = TokenMapper::new();

        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser();

        let mut instance = CompanyTokenMapper {
            token_mapper,
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            reverse_ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            company_token_sequences_map: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        };

        instance.ingest_company_tokens(&company_symbol_list);

        instance
    }

    fn clear(&mut self) {
        self.company_token_sequences_map.clear();
        self.company_reverse_token_map.clear();
        self.ticker_symbol_map.clear();
        self.reverse_ticker_symbol_map.clear();
    }

    /// Ingests tokens from the company symbol list
    fn ingest_company_tokens(&mut self, company_symbol_list: &CompanySymbolList) {
        self.clear();

        for (ticker_symbol, company_name, alt_company_names) in company_symbol_list {
            // let company_name_key = company_name.clone().unwrap();

            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                let ticker_symbol_token_id = self.token_mapper.upsert_token(&ticker_symbol_token);

                self.ticker_symbol_map
                    .insert(ticker_symbol.clone(), ticker_symbol_token_id);

                self.reverse_ticker_symbol_map
                    .insert(ticker_symbol_token_id, ticker_symbol.clone());
            }

            // TODO: Don't use unwrap
            // FIXME: This should use get_ticker_symbol_token_id instead but it's problematic here
            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            let ticker_symbol_token_id = self
                .token_mapper
                .get_token_id(&ticker_symbol_tokens[0])
                .unwrap();

            if let Some(company_name) = company_name {
                let company_name_token_ids = self.process_company_name_tokens(&company_name);
                all_company_name_token_ids.push(company_name_token_ids.clone());

                // Populate reverse map
                for token_id in company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id.clone())
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol_token_id);
                }
            }

            // Process alternate company names
            for alt_company_name in alt_company_names {
                let alt_company_name_token_ids =
                    self.process_company_name_tokens(&alt_company_name);
                all_company_name_token_ids.push(alt_company_name_token_ids.clone());

                // Populate reverse map
                for token_id in alt_company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol_token_id);
                }
            }

            // Insert the collected token IDs into the map
            self.company_token_sequences_map
                .entry(ticker_symbol_token_id)
                .or_insert_with(Vec::new)
                .extend(all_company_name_token_ids);

            // TODO: Remove
            // println!(
            //     "sequences map length: {}",
            //     self.company_reverse_token_map.len()
            // );
        }
    }

    /// Helper method for per-company token ingestion
    fn process_company_name_tokens(&mut self, company_name: &str) -> Vec<TokenId> {
        let company_name_tokens = self.text_doc_tokenizer.tokenize(&company_name);
        let mut company_name_token_ids = Vec::new();
        for token in company_name_tokens {
            let token_id = self.token_mapper.upsert_token(&token);
            company_name_token_ids.push(token_id);
        }

        company_name_token_ids
    }

    // TODO: Use actual error type (or return option type)
    pub fn get_ticker_symbol_by_token_id(
        &self,
        token_id: &TokenId,
    ) -> Result<&TickerSymbol, String> {
        // TODO: Remove
        // println!("{:?}", self.reverse_ticker_symbol_map);

        match self.reverse_ticker_symbol_map.get(token_id) {
            Some(ticker_symbol) => Ok(ticker_symbol),
            None => Err(format!("Could not ticker from token token id: {}", token_id).to_string()),
        }
    }

    // TODO: Use actual error type (or return option type)
    pub fn get_ticker_symbol_token_id(
        &self,
        ticker_symbol: &TickerSymbol,
    ) -> Result<&TokenId, String> {
        match self.ticker_symbol_map.get(ticker_symbol) {
            Some(token_id) => Ok(token_id),
            None => Err("Could not obtain token id".to_string()),
        }
    }

    pub fn get_company_token_sequence_max_length(
        &self,
        ticker_symbol_token_id: &TickerSymbolTokenId,
        company_sequence_idx: CompanySequenceIndex,
    ) -> Option<usize> {
        self.company_token_sequences_map
            .get(&ticker_symbol_token_id)
            .and_then(|seq| seq.get(company_sequence_idx).map(|s| s.len()))
    }
}
