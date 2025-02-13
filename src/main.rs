use log::error;
use std::io::{self, Read};
use ticker_sniffer::{extract_tickers_from_text, sort_results};

fn main() {
    // Initialize the logger
    #[cfg(feature = "logger-support")]
    {
        env_logger::init();
    }

    // Read the input text from stdin
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        error!("Failed to read from stdin: {}", e);
        std::process::exit(1);
    }

    // Extract tickers from the input text
    match extract_tickers_from_text(&input, true) {
        Ok(results) => {
            let sorted_results = sort_results(results);

            for (ticker_symbol, frequency) in sorted_results {
                println!("{}: {:.2}", ticker_symbol, frequency);
            }
        }
        Err(e) => {
            error!("Error extracting tickers: {}", e);
            std::process::exit(1);
        }
    }
}
