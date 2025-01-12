mod config;
pub use config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
mod constants;
pub mod models;
pub use models::{
    CompanyTokenProcessor, CompanyTokenProcessorConfig, Error, TokenMapper, TokenParityState,
    TokenRangeState, Tokenizer,
};
pub mod types;
mod utils;
pub use types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    Token, TokenId, TokenRef, TokenVector,
};

// TODO: Add dedicated type instead of f32
pub fn extract_tickers_from_text(
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<TickerSymbolFrequencyMap, Error> {
    let results_ticker_symbol_frequency_map = extract_tickers_from_text_with_custom_config(
        DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
        &text,
        &company_symbols_list,
    )?;

    Ok(results_ticker_symbol_frequency_map)
}

// TODO: Refactor accordingly
pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<TickerSymbolFrequencyMap, Error> {
    let mut company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, company_symbols_list);

    let results_ticker_symbol_frequency_map = company_token_processor.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
