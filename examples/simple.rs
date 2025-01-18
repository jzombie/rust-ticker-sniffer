#![allow(dead_code, unused_imports, unused_variables)]

use ticker_sniffer::extract_tickers_from_text;

use log::{debug, error, info, log_enabled, Level};

use std::collections::HashMap;

fn main() {
    env_logger::init();

    let text = "Berkshire Hathaway and Apple";

    let results = extract_tickers_from_text(&text).unwrap();

    println!("Extracted Tickers for the given text \"{}\"", text);
    for (ticker_symbol, frequency) in results {
        println!("{}: {:.2}", ticker_symbol, frequency);
    }
}
