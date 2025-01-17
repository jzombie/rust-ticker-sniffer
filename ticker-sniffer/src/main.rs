#![allow(dead_code, unused_imports, unused_variables)]

use log::{error, info};
use std::collections::HashMap;
use std::io::{self, Read};
use ticker_sniffer::extract_tickers_from_text;

fn main() {
    // Initialize the logger
    env_logger::init();

    // Read the input text from stdin
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        error!("Failed to read from stdin: {}", e);
        std::process::exit(1);
    }

    // Extract tickers from the input text
    match extract_tickers_from_text(&input) {
        Ok(results) => {
            println!("Extracted Tickers:");
            for (ticker_symbol, frequency) in results {
                println!("{}: {:.2}", ticker_symbol, frequency);
            }
        }
        Err(e) => {
            error!("Error extracting tickers: {}", e);
            std::process::exit(1);
        }
    }
}
