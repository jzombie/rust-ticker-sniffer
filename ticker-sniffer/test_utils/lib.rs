use std::{fs, path::Path};
use ticker_sniffer::{
    extract_tickers_from_text_with_custom_config, CompanyTokenProcessorConfig, Error as LibError,
    TickerSymbol, TickerSymbolFrequencyMap,
};
// pub use models::EvaluationResult;
pub mod constants;

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
    company_token_processor_config: &CompanyTokenProcessorConfig,
) -> Result<
    (
        TickerSymbolFrequencyMap,
        Vec<TickerSymbol>,
        Vec<TickerSymbol>,
    ),
    LibError,
> {
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
    let results_ticker_symbol_frequency_map = extract_tickers_from_text_with_custom_config(
        &company_token_processor_config,
        &filtered_text,
    )?;

    // Get the expected tickers from the file
    let expected_tickers = get_expected_tickers(&Path::new(test_file_path));

    // Separate actual results into a vector of just tickers
    let actual_tickers: Vec<TickerSymbol> = results_ticker_symbol_frequency_map
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
    Ok((
        results_ticker_symbol_frequency_map,
        unexpected_tickers,
        missing_tickers,
    ))
}
