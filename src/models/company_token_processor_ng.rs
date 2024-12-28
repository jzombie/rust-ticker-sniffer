use crate::types::CompanySymbolList;
use crate::TokenMapper;
use crate::Tokenizer;

// TODO: Rename without the `Ng` suffix
pub struct CompanyTokenProcessorNg<'a> {
    pub company_symbol_list: &'a CompanySymbolList,
    pub token_mapper: TokenMapper,
    pub ticker_symbol_tokenizer: Tokenizer,
    pub text_doc_tokenizer: Tokenizer,
}

impl<'a> CompanyTokenProcessorNg<'a> {
    pub fn new(company_symbol_list: &'a CompanySymbolList) -> Self {
        let mut instance = CompanyTokenProcessorNg {
            company_symbol_list,
            token_mapper: TokenMapper::new(),
            ticker_symbol_tokenizer: Tokenizer::ticker_symbol_parser(),
            text_doc_tokenizer: Tokenizer::text_doc_parser(),
        };

        instance.ingest_company_tokens();

        instance
    }

    fn ingest_company_tokens(&mut self) {
        for (ticker_symbol, company_name, alternate_company_names) in self.company_symbol_list {
            // println!("{}", ticker_symbol);

            // Workaround for "urban-gro, Inc."
            // The tokenizer filters on words with uppercase letters, which this does not have
            // let uc_company_name = company_name.clone().unwrap().to_uppercase();

            let ticker_symbol_tokens = self.ticker_symbol_tokenizer.tokenize(&ticker_symbol);
            for ticker_symbol_token in ticker_symbol_tokens {
                self.token_mapper.upsert_token(&ticker_symbol_token);
            }

            let company_name_tokens = self
                .text_doc_tokenizer
                .tokenize(&company_name.clone().unwrap());
            for token in company_name_tokens {
                // TODO: Retain map of company name token associations
                self.token_mapper.upsert_token(&token);
            }

            for alternate_company_name in alternate_company_names {
                // let uc_alternate_name = alternate_company_name.clone().to_uppercase();
                let alternate_company_name_tokens = self
                    .text_doc_tokenizer
                    .tokenize(&alternate_company_name.clone());

                for token in alternate_company_name_tokens {
                    // TODO: Retain map of alternate company name token associations
                    self.token_mapper.upsert_token(&token);
                }
            }
        }
    }

    pub fn process_text_doc(&mut self, text: &str) {
        let text_doc_tokens = self.text_doc_tokenizer.tokenize(text);
        let filtered_tokens = self
            .token_mapper
            .get_filtered_tokens(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        // TODO: Remove
        println!("Text doc tokens: {:?}", text_doc_tokens);
        println!("Filtered tokens: {:?}", filtered_tokens,);
    }

    // pub fn process_company_name(&mut self, company_name: &str) -> Vec<usize> {
    //     let tokens = self.tokenizer.tokenize(company_name);
    //     tokens
    //         .iter()
    //         .map(|token| self.token_mapper.upsert_token(token))
    //         .collect()
    // }

    // pub fn get_token_id(&self, token: &str) -> Option<usize> {
    //     self.token_mapper.get_token_id(token)
    // }

    // pub fn get_token_count(&self) -> usize {
    //     self.token_mapper.token_count()
    // }
}
