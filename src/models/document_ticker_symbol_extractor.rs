use crate::{CompanyTokenProcessor, Tokenizer};

pub struct DocumentTickerSymbolExtractor<'a> {
    company_token_processor: CompanyTokenProcessor<'a>,
}
