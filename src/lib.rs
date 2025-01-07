mod constants;
pub mod models;
pub use constants::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
pub use models::{
    CompanyTokenProcessor, CompanyTokenProcessorConfig, Error, TokenMapper, Tokenizer,
};
pub mod types;
mod utils;
pub use types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, Token, TokenId, TokenRef,
    TokenVector,
};

// TODO: Add dedicated type instead of f32
pub fn extract_tickers_from_text(
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<Vec<(TickerSymbol, f32)>, Error> {
    let symbols_with_confidence = extract_tickers_from_text_with_custom_config(
        DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
        &text,
        &company_symbols_list,
    )?;

    Ok(symbols_with_confidence)
}

// TODO: Refactor accordingly
pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<Vec<(TickerSymbol, f32)>, Error> {
    let mut company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, company_symbols_list);
    company_token_processor.process_text_doc(text);

    // TODO: Remove mock
    let symbols_with_confidence = vec![];

    Ok(symbols_with_confidence)
}
