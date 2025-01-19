mod config;
pub mod constants;
mod utils;
pub use config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
pub mod models;
pub use models::{
    CompanySymbolListPreprocessor, CompanyTokenMapper, CompanyTokenProcessor,
    CompanyTokenProcessorConfig, Error, TokenMapper, TokenParityState, TokenRangeState, Tokenizer,
};

pub mod types;
pub use types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    Token, TokenId, TokenRef, TokenVector,
};

// Note: Ideally this string would not be hardcoded, but `include_bytes!` requires a string literal.
const COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY: &[u8] =
    include_bytes!("../__AUTOGEN__company_symbol_list.csv.gz");

/// Extracts ticker symbols from the provided text using the default configuration.
///
/// # Arguments
/// * `text` - A reference to the input text document from which ticker symbols
///   are to be extracted.
///
/// # Returns
/// * `Ok(TickerSymbolFrequencyMap)` - A map of ticker symbols and their
///   frequencies if the operation is successful.
/// * `Err(Error)` - An error if processing fails.
///
/// # Example
/// ```
/// let text = "AAPL and MSFT are leading companies.";
/// let result = extract_tickers_from_text(text);
/// assert!(result.is_ok());
/// ```
pub fn extract_tickers_from_text(text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
    let results_ticker_symbol_frequency_map =
        extract_tickers_from_text_with_custom_config(DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG, text)?;

    Ok(results_ticker_symbol_frequency_map)
}

/// Extracts ticker symbols from the provided text using a custom configuration.
///
/// # Arguments
/// * `document_token_processor_config` - A reference to the custom configuration
///   for processing tokens.
/// * `text` - A reference to the input text document from which ticker symbols
///   are to be extracted.
///
/// # Returns
/// * `Ok(TickerSymbolFrequencyMap)` - A map of ticker symbols and their
///   frequencies if the operation is successful.
/// * `Err(Error)` - An error if processing fails.
///
/// # Example
/// ```
/// let config = CompanyTokenProcessorConfig::default();
/// let text = "GOOGL is a tech giant.";
/// let result = extract_tickers_from_text_with_custom_config(&config, text);
/// assert!(result.is_ok());
/// ```
pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Load the company symbol list
    let company_symbol_list =
        CompanySymbolListPreprocessor::extract_company_symbol_list_from_bytes(
            COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY,
        )?;

    let company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, &company_symbol_list);

    let results_ticker_symbol_frequency_map = company_token_processor?.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
