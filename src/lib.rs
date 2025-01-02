mod constants;
pub mod models;
pub use constants::DEFAULT_COMPANY_NAME_EXTRACTOR_CONFIG;
pub use models::{
    CompanyTokenProcessor, DocumentCompanyNameExtractorConfig, Error, TokenMapper, Tokenizer,
};
pub mod types;
mod utils;
pub use types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TokenId, TokenVector,
};

// TODO: Add dedicated type instead of f32
pub fn extract_tickers_from_text(
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<Vec<(TickerSymbol, f32)>, Error> {
    let symbols_with_confidence = extract_tickers_from_text_with_custom_config(
        &text,
        &company_symbols_list,
        DEFAULT_COMPANY_NAME_EXTRACTOR_CONFIG,
    )?;

    Ok(symbols_with_confidence)
}

// TODO: Refactor accordingly
pub fn extract_tickers_from_text_with_custom_config(
    text: &str,
    company_symbols_list: &CompanySymbolList,
    document_company_name_extractor_config: DocumentCompanyNameExtractorConfig,
) -> Result<Vec<(TickerSymbol, f32)>, Error> {
    let mut company_token_processor = CompanyTokenProcessor::new(company_symbols_list);
    company_token_processor.process_text_doc(text);

    // TODO: Remove mock
    let symbols_with_confidence = vec![];

    Ok(symbols_with_confidence)
}
