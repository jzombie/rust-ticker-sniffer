use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::{fs, path::Path};
use ticker_sniffer::{
    extract_tickers_from_text_with_custom_weights, CompanySymbolList,
    DocumentCompanyNameExtractorConfig, Error as LibError, TickerSymbol,
};
pub mod models;
// pub use models::EvaluationResult;
pub mod constants;
use constants::TEST_SYMBOLS_CSV_PATH;

/// Utility to load symbols from a CSV file for testing and benchmarking.
pub fn load_company_symbol_list_from_file(
    file_path: &str,
) -> Result<CompanySymbolList, Box<dyn Error>> {
    let mut company_symbols_list = CompanySymbolList::new();
    let mut reader = Reader::from_path(file_path)?;

    // Use headers to extract columns
    let headers = reader.headers()?.clone();

    for record in reader.records() {
        let record = record?;
        // Extract values based on header names
        let symbol = record.get(headers.iter().position(|h| h == "Symbol").unwrap());
        let company_name = record.get(headers.iter().position(|h| h == "Company Name").unwrap());

        if let Some(symbol) = symbol {
            company_symbols_list.push((
                symbol.to_uppercase(),
                company_name.map(|name| name.to_string()),
            ));
        } else {
            eprintln!("Skipping invalid row: {:?}", record);
        }
    }

    Ok(company_symbols_list)
}

// Helper function to get the expected tickers from the text file
pub fn get_expected_tickers(file_path: &Path) -> Vec<TickerSymbol> {
    // Read the content of the text file
    let content = fs::read_to_string(file_path).expect("Failed to read test file");

    // Extract tickers from lines starting with EXPECTED:
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("EXPECTED:") {
                Some(line.replace("EXPECTED:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

// Helper function to check if the file has an EXPECTED_FAILURE line
pub fn get_expected_failure(file_path: &Path) -> Option<TickerSymbol> {
    let content = fs::read_to_string(file_path).expect("Failed to read test file");

    content.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with("EXPECTED_FAILURE:") {
            Some(line.replace("EXPECTED_FAILURE:", "").trim().to_string())
        } else {
            None
        }
    })
}

// Helper function to run the test for each file in the directory
pub fn run_test_for_file(
    test_file_path: &str,
    company_name_extractor_config: DocumentCompanyNameExtractorConfig,
) -> Result<
    (
        // TODO: Use more specific types (especially for `f32` confidence score)
        Vec<(TickerSymbol, f32)>,
        Vec<TickerSymbol>,
        Vec<TickerSymbol>,
    ),
    LibError,
> {
    // Load symbols from a test CSV file
    let symbols_map = load_company_symbol_list_from_file(TEST_SYMBOLS_CSV_PATH)
        .expect("Failed to load symbols from CSV");

    // Read the content of the text file
    let raw_text = fs::read_to_string(test_file_path).expect("Failed to read test file");

    // Filter out lines starting with 'EXPECTED:', 'EXPECTED_FAILURE:', or 'COMMENT:'
    let filtered_text: String = raw_text
        .lines()
        .filter(|line| {
            !line.trim_start().starts_with("EXPECTED:")
                && !line.trim_start().starts_with("EXPECTED_FAILURE:")
                && !line.trim_start().starts_with("COMMENT:")
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // Extract tickers from the filtered text
    let results_with_confidence = extract_tickers_from_text_with_custom_weights(
        &filtered_text,
        &symbols_map,
        company_name_extractor_config,
    )?;

    // Get the expected tickers from the file
    let expected_tickers = get_expected_tickers(&Path::new(test_file_path));

    // Separate actual results into a vector of just tickers
    let actual_tickers: Vec<TickerSymbol> = results_with_confidence
        .iter()
        .map(|(symbol, _confidence)| symbol)
        .cloned()
        .collect();

    // Determine unexpected and missing tickers
    let unexpected_tickers: Vec<TickerSymbol> = actual_tickers
        .iter()
        .filter(|ticker| !expected_tickers.contains(ticker))
        .cloned()
        .collect();

    let missing_tickers: Vec<TickerSymbol> = expected_tickers
        .iter()
        .filter(|ticker| !actual_tickers.contains(ticker))
        .cloned()
        .collect();

    // Assertions for correctness
    assert_eq!(
        actual_tickers.len(),
        expected_tickers.len(),
        "{} - Expected {} tickers but found {}. Missing: {:?}, Unexpected: {:?}",
        test_file_path,
        expected_tickers.len(),
        actual_tickers.len(),
        missing_tickers,
        unexpected_tickers
    );

    for ticker in &expected_tickers {
        assert!(
            actual_tickers.contains(ticker),
            "{} - Expected ticker {:?} was not found in results. Found: {:?}",
            test_file_path,
            ticker,
            actual_tickers
        );
    }

    for ticker in &unexpected_tickers {
        assert!(
            !expected_tickers.contains(ticker),
            "{} - Unexpected ticker {:?} found in results.",
            test_file_path,
            ticker
        );
    }

    // Return the results along with the lists of unexpected and missing tickers
    Ok((results_with_confidence, unexpected_tickers, missing_tickers))
}
