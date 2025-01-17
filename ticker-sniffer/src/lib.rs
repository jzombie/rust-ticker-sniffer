mod config;
mod utils;
pub use config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
pub mod models;
pub use models::{
    CompanyTokenProcessor, CompanyTokenProcessorConfig, TokenParityState, TokenRangeState,
};
pub use ticker_sniffer_common_lib::models::{
    CompanySymbolListPreprocessor, CompanyTokenMapper, Error, TokenMapper, Tokenizer,
};
pub use ticker_sniffer_common_lib::types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    Token, TokenId, TokenRef, TokenVector,
};

const COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY: &[u8] =
    include_bytes!("../__AUTOGEN__company_symbol_list.csv.gz");

pub fn extract_tickers_from_text(text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
    let results_ticker_symbol_frequency_map =
        extract_tickers_from_text_with_custom_config(DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG, text)?;

    Ok(results_ticker_symbol_frequency_map)
}

pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Load the company symbol list
    let company_symbol_list =
        CompanySymbolListPreprocessor::extract_company_symbol_list_from_bytes(
            &COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY,
        )?;

    let company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, &company_symbol_list);

    let results_ticker_symbol_frequency_map = company_token_processor?.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
