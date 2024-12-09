use crate::types::CompanySymbolsList;
use utils::CompanyTokenProcessor;

pub struct TickerExtractorWeights {
    min_similarity_threshold: f32,
    token_length_diff_tolerance: usize,
}

pub struct TickerExtractor {
    company_token_processor: CompanyTokenProcessor,
    text: String,
    weights: TickerExtractorWeights,
    tokenized_query_vectors: Vec<Vec<u32>>,
    results: Vec<TickerSymbol>,
}

impl TickerExtractor {
    pub fn new(company_symbols_list: &CompanySymbolsList, text: &str) -> Self {
        let company_token_processor = CompanyTokenProcessor::new(&company_symbols_list);

        Self {
            company_token_processor,
            text,
            // TODO: Apply default weights
            weights: TickerExtractorWeights {
                min_similarity_threshold: 0.0,
                token_length_diff_tolerance: 0,
            },
            tokenized_query_vectors: vec![],
            results: vec![],
        }
    }

    pub fn extract(self, text: &str) {
        self.text = text;
    }
}
