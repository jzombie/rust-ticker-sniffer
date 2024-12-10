#![allow(dead_code, unused_imports, unused_variables)]

#[path = "../../test_utils/lib.rs"]
mod test_utils;
use test_utils::constants::TEST_SYMBOLS_CSV_PATH;
use test_utils::load_symbols_from_file;
use ticker_sniffer::extract_tickers_from_text;

use std::collections::HashMap;

fn main() {
    let company_symbols_list =
        load_symbols_from_file(TEST_SYMBOLS_CSV_PATH).expect("Failed to load symbols from CSV");

    println!(
        "{:?}",
        // extract_tickers_from_text(&"Nvidia Corporation", &company_symbols_list)
        // extract_tickers_from_text(&"Nvidia", &company_symbols_list)
        extract_tickers_from_text(
            &"The shift was prompted by Wal-mart's...",
            &company_symbols_list
        )
    )
}
