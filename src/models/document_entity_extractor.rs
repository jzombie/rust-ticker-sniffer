use crate::types::CompanySymbolsList;
use crate::types::TickerSymbol;
use crate::{DocumentCompanyNameExtractor, DocumentCompanyNameExtractorConfig};
use std::collections::HashMap;

pub struct DocumentEntityExtractor<'a> {
    document_company_name_extractor: DocumentCompanyNameExtractor<'a>,
}

impl<'a> DocumentEntityExtractor<'a> {
    pub fn new(
        company_symbols_list: &'a CompanySymbolsList,
        document_company_name_extractor_config: DocumentCompanyNameExtractorConfig,
    ) -> Self {
        let document_company_name_extractor = DocumentCompanyNameExtractor::new(
            company_symbols_list,
            document_company_name_extractor_config,
        );

        Self {
            document_company_name_extractor,
        }
    }

    pub fn extract(&mut self, text: &str) -> (HashMap<TickerSymbol, f32>, Vec<usize>) {
        self.document_company_name_extractor.extract(text)
    }
}
