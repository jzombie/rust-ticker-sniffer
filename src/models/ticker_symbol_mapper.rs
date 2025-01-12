use std::collections::HashMap;

use crate::types::{
    CompanySymbolList, CompanyTokenSequencesMap, ReverseTickerSymbolMap, TickerSymbol,
    TickerSymbolMap, TokenId,
};

use crate::{TokenMapper, Tokenizer};

// TODO: Precompute during compile time, not run time
pub struct TickerSymbolMapper<'a> {
    pub company_symbol_list: &'a CompanySymbolList,
    pub ticker_symbol_map: TickerSymbolMap,
    pub reverse_ticker_symbol_map: ReverseTickerSymbolMap,
    // TODO: Replace tickersymbol with a token ID representing the ticker
    // symbol, and use the reverse ticker symbol map to map them back?
    pub company_token_sequences: CompanyTokenSequencesMap,
    pub company_reverse_token_map: HashMap<TokenId, Vec<TickerSymbol>>,
}

impl<'a> TickerSymbolMapper<'a> {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        TickerSymbolMapper {
            company_symbol_list,
            ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            reverse_ticker_symbol_map: HashMap::with_capacity(company_symbol_list.len()),
            company_token_sequences: HashMap::with_capacity(company_symbol_list.len()),
            company_reverse_token_map: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.company_token_sequences.clear();
        self.company_reverse_token_map.clear();
        self.ticker_symbol_map.clear();
        self.reverse_ticker_symbol_map.clear();
    }

    /// Ingests tokens from the company symbol list
    pub fn ingest_company_tokens(
        &mut self,
        company_symbol_list: &'a CompanySymbolList,
        ticker_symbol_tokenizer: &Tokenizer,
        text_doc_tokenizer: &Tokenizer,
        token_mapper: &mut TokenMapper,
    ) {
        self.clear();

        for (ticker_symbol, company_name, alt_company_names) in company_symbol_list {
            // let company_name_key = company_name.clone().unwrap();

            let mut all_company_name_token_ids = Vec::new();

            // Tokenize the ticker symbol and upsert token IDs
            let ticker_symbol_tokens = ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                let ticker_symbol_token_id = token_mapper.upsert_token(&ticker_symbol_token);

                self.ticker_symbol_map
                    .insert(ticker_symbol.clone(), ticker_symbol_token_id);

                self.reverse_ticker_symbol_map
                    .insert(ticker_symbol_token_id, ticker_symbol.clone());
            }

            if let Some(company_name) = company_name {
                let company_name_token_ids = self.process_company_name_tokens(
                    &company_name,
                    &text_doc_tokenizer,
                    token_mapper,
                );
                all_company_name_token_ids.push(company_name_token_ids.clone());

                // Populate reverse map
                for token_id in company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id.clone())
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol.clone());
                }
            }

            // Process alternate company names
            for alt_company_name in alt_company_names {
                let alt_company_name_token_ids = self.process_company_name_tokens(
                    &alt_company_name,
                    &text_doc_tokenizer,
                    token_mapper,
                );
                all_company_name_token_ids.push(alt_company_name_token_ids.clone());

                // Populate reverse map
                for token_id in alt_company_name_token_ids {
                    self.company_reverse_token_map
                        .entry(token_id)
                        .or_insert_with(Vec::new)
                        .push(ticker_symbol.clone());
                }
            }

            // Insert the collected token IDs into the map
            self.company_token_sequences
                .entry(ticker_symbol.clone())
                .or_insert_with(Vec::new)
                .extend(all_company_name_token_ids);
        }
    }

    /// Helper method for per-company token ingestion
    fn process_company_name_tokens(
        &mut self,
        company_name: &str,
        text_doc_tokenizer: &Tokenizer,
        token_mapper: &mut TokenMapper,
    ) -> Vec<TokenId> {
        let company_name_tokens = text_doc_tokenizer.tokenize(&company_name);
        let mut company_name_token_ids = Vec::new();
        for token in company_name_tokens {
            let token_id = token_mapper.upsert_token(&token);
            company_name_token_ids.push(token_id);
        }

        company_name_token_ids
    }
}
