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

use flate2::read::GzDecoder;
use std::io::Read;

mod utils;

const COMPRESSED_COMPANY_SYMBOL_LIST: &[u8] =
    include_bytes!("../__AUTOGEN__company_symbol_list.csv.gz");

// TODO: Refactor
/// Decompress and parse the company symbol list from the embedded Gzip file
fn load_company_symbol_list() -> Result<CompanySymbolList, Error> {
    // Decompress the Gzip file
    let mut decoder = GzDecoder::new(COMPRESSED_COMPANY_SYMBOL_LIST);
    let mut decompressed_data = String::new();
    decoder.read_to_string(&mut decompressed_data)?;

    // Use the utility function to parse the CSV data
    let company_symbol_list = utils::read_company_symbol_list_from_string(&decompressed_data)?;
    Ok(company_symbol_list)
}

// TODO: Refactor
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
    // Load the company symbol list
    let company_symbol_list = load_company_symbol_list()?;

    let mut company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, &company_symbol_list);

    let results_ticker_symbol_frequency_map = company_token_processor.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
