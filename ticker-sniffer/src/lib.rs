mod config;
pub use config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
pub mod models;
pub use models::{
    CompanyTokenProcessor, CompanyTokenProcessorConfig, TokenParityState, TokenRangeState,
};
pub use ticker_sniffer_common_lib::models::{CompanyTokenMapper, Error, TokenMapper, Tokenizer};
pub use ticker_sniffer_common_lib::types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    Token, TokenId, TokenRef, TokenVector,
};

mod utils;

pub fn extract_tickers_from_text(text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
    let results_ticker_symbol_frequency_map = extract_tickers_from_text_with_custom_config(
        DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
        &text,
    )?;

    Ok(results_ticker_symbol_frequency_map)
}

pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
) -> Result<TickerSymbolFrequencyMap, Error> {
    let mut company_token_processor =
        CompanyTokenProcessor::from_prebuilt(document_token_processor_config);

    let results_ticker_symbol_frequency_map = company_token_processor.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
