#[cfg(doctest)]
doc_comment::doctest!("../README.md");

pub mod config;
pub use config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
pub mod constants;
pub mod structs;
mod utils;
pub use structs::{
    CompanySymbolListPreprocessor, CompanyTokenMapper, CompanyTokenProcessor,
    CompanyTokenProcessorConfig, Error, TokenMapper, TokenParityState, TokenRangeState, Tokenizer,
};
pub use utils::sort_results;
pub mod types;
pub use types::{
    AlternateCompanyName, CompanyName, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    Token, TokenId, TokenRef, TokenVector,
};
use structs::ResourceContainer;


/// Extracts ticker symbols from the provided text using the default configuration.
///
/// # Arguments
/// * `text` - A reference to the input text document from which ticker symbols
///   are to be extracted.
/// * `is_case_sensitive` - Whether or not the text document should be filtered using case sensitivity.
///
/// # Returns
/// * `Ok(TickerSymbolFrequencyMap)` - A map of ticker symbols and their
///   frequencies if the operation is successful.
/// * `Err(Error)` - An error if processing fails.
///
/// # Example
/// ```
/// use ticker_sniffer::extract_tickers_from_text;
///
/// let text = "Apple and Microsoft are leading companies.";
/// let result = extract_tickers_from_text(text, true);
/// assert!(result.is_ok());
/// ```
pub fn extract_tickers_from_text(
    text: &str,
    is_case_sensitive: bool,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Skip entirely if there is no text
    if text.is_empty() {
        return Ok(TickerSymbolFrequencyMap::new());
    }

    let results_ticker_symbol_frequency_map = extract_tickers_from_text_with_custom_config(
        DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
        text,
        is_case_sensitive,
    )?;

    Ok(results_ticker_symbol_frequency_map)
}

/// Extracts ticker symbols from the provided text using a custom configuration.
///
/// # Arguments
/// * `document_token_processor_config` - A reference to the custom configuration
///   for processing tokens.
/// * `text` - A reference to the input text document from which ticker symbols
///   are to be extracted.
/// * `is_case_sensitive` - Whether or not the text document should be filtered using case sensitivity.
///
/// # Returns
/// * `Ok(TickerSymbolFrequencyMap)` - A map of ticker symbols and their
///   frequencies if the operation is successful.
/// * `Err(Error)` - An error if processing fails.
///
/// # Example
/// ```
/// use ticker_sniffer::config::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
/// use ticker_sniffer::extract_tickers_from_text_with_custom_config;
///
/// let config = DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;
/// let text = "Google is a tech giant.";
/// let result = extract_tickers_from_text_with_custom_config(&config, text, true);
/// assert!(result.is_ok());
/// ```
pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
    is_case_sensitive: bool,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Load the company symbol list
    let company_symbol_list =
        CompanySymbolListPreprocessor::extract_company_symbol_list_from_bytes(
            ResourceContainer::COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY,
        )?;

    let company_token_processor = CompanyTokenProcessor::new(
        document_token_processor_config,
        &company_symbol_list,
        is_case_sensitive,
    );

    let results_ticker_symbol_frequency_map = company_token_processor?.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
