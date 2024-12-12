use crate::types::{CompanySymbolList, TickerSymbol};
use crate::{
    CompanyTokenProcessor, DocumentCompanyNameExtractor, DocumentCompanyNameExtractorConfig,
    Tokenizer,
};
use std::collections::HashMap;

pub struct DocumentEntityExtractor {}

impl DocumentEntityExtractor {
    pub fn extract(
        company_symbols_list: &CompanySymbolList,
        document_company_name_extractor_config: &DocumentCompanyNameExtractorConfig,
        text: &str,
    ) -> (HashMap<TickerSymbol, f32>, Vec<usize>) {
        let text_doc_tokenizer = Tokenizer::text_doc_parser();
        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();

        let company_token_processor = CompanyTokenProcessor::new(
            &company_symbols_list,
            &ticker_symbol_tokenizer,
            &text_doc_tokenizer,
        );

        let mut document_company_name_extractor = DocumentCompanyNameExtractor::new(
            &company_symbols_list,
            &document_company_name_extractor_config,
            &text_doc_tokenizer,
            &company_token_processor,
        );

        document_company_name_extractor.extract(text)
    }
}
