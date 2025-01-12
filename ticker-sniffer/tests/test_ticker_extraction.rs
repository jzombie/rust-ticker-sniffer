#[path = "../test_utils/lib.rs"]
mod test_utils;
use test_utils::constants::TEST_FILES_DIRECTORY;

use std::fs::read_dir;
use test_utils::run_test_for_file;
use ticker_sniffer::DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_from_multiple_files() {
        println!("Testing ticker extractions...");

        // Directory containing the test files
        let test_dir = TEST_FILES_DIRECTORY;

        // Read all files in the directory
        let files = read_dir(test_dir)
            .expect("Failed to read test files directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect directory entries");

        let total_files = files.len();

        for (file_idx, file) in files.iter().enumerate() {
            let file_path = file.path();

            println!(
                "   -- {:?} ({} of {})",
                file_path,
                file_idx + 1,
                total_files
            );

            // Run the test for each file (if it is a file)
            if file_path.is_file() {
                let _ = run_test_for_file(
                    file_path.to_str().unwrap(),
                    DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
                );
            }
        }
    }
}
