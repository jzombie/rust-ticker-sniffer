use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

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

// LazyLock for dynamically determining the file path at runtime
static COMPRESSED_COMPANY_SYMBOL_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");
    PathBuf::from(out_dir).join("__AUTOGEN__company_symbol_list.csv.gz")
});

pub fn extract_tickers_from_text(text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
    let results_ticker_symbol_frequency_map =
        extract_tickers_from_text_with_custom_config(DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG, text)?;

    Ok(results_ticker_symbol_frequency_map)
}

pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Load the compressed company symbol list at runtime
    let compressed_data = load_compressed_company_symbol_list()?;

    let company_symbol_list =
        CompanySymbolListPreprocessor::extract_company_symbol_list_from_bytes(&compressed_data)?;

    let company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, &company_symbol_list);

    let results_ticker_symbol_frequency_map = company_token_processor?.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}

// Function to load the compressed file from OUT_DIR
fn load_compressed_company_symbol_list() -> Result<Vec<u8>, std::io::Error> {
    let file_path = &*COMPRESSED_COMPANY_SYMBOL_FILE_PATH;
    fs::read(file_path)
}
