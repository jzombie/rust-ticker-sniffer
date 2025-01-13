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
use bincode;
use serde::{Deserialize, Serialize};

// Embed the bytes from the pre-generated binary file
const DUMMY_GENERATED_BYTES: &[u8] = include_bytes!("./__dummy_generated__.bin");

#[derive(Serialize, Deserialize, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct World(Vec<Entity>);

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

pub fn extract_tickers_from_text_with_custom_config(
    document_token_processor_config: &CompanyTokenProcessorConfig,
    text: &str,
    company_symbols_list: &CompanySymbolList,
) -> Result<TickerSymbolFrequencyMap, Error> {
    // Deserialize the embedded bytes
    match bincode::deserialize::<World>(DUMMY_GENERATED_BYTES) {
        Ok(world) => {
            println!("Deserialized World: {:?}", world);
        }
        Err(err) => {
            eprintln!("Failed to deserialize embedded data: {}", err);
        }
    }

    let mut company_token_processor =
        CompanyTokenProcessor::new(document_token_processor_config, company_symbols_list);

    let results_ticker_symbol_frequency_map = company_token_processor.process_text_doc(text)?;

    Ok(results_ticker_symbol_frequency_map)
}
