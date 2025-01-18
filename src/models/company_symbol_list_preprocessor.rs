use crate::types::CompanySymbolList;
use crate::Error;
use csv::ReaderBuilder;
use flate2::read::GzDecoder;
use std::io::Cursor;
use std::io::Read;

pub struct CompanySymbolListPreprocessor {}

/// A preprocessor for the company symbol list, providing utilities for decompressing and
/// parsing the embedded data.
impl CompanySymbolListPreprocessor {
    /// Decompress and parse the company symbol list from an embedded Gzip file.
    ///
    /// # Arguments
    ///
    /// * `read_bytes` - A slice of bytes representing the compressed Gzip data.
    ///
    /// # Errors
    ///
    /// Returns an error if decompression or parsing fails (e.g., invalid Gzip format
    /// or malformed CSV data).
    pub fn extract_company_symbol_list_from_bytes(
        read_bytes: &[u8],
    ) -> Result<CompanySymbolList, Error> {
        // Decompress the Gzip file
        let mut decoder = GzDecoder::new(read_bytes);
        let mut decompressed_data = String::new();
        decoder.read_to_string(&mut decompressed_data)?;

        // Use the utility function to parse the CSV data
        let company_symbol_list = Self::read_company_symbol_list_from_string(&decompressed_data)?;
        Ok(company_symbol_list)
    }

    /// Parse a company symbol list from a CSV-formatted string.
    ///
    /// # Arguments
    ///
    /// * `csv_str` - A reference to a string containing the CSV data.
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV reader encounters issues (e.g., invalid
    /// format, missing headers, or missing data).
    ///
    /// # Notes
    ///
    /// This function expects the CSV to have the following column headers:
    /// - `Symbol`: The primary symbol for the company.
    /// - `Company Name`: The name of the company (optional).
    /// - `Alternate Names`: A comma-separated list of alternate names (optional).
    pub fn read_company_symbol_list_from_string(csv_str: &str) -> Result<CompanySymbolList, Error> {
        let mut company_symbols_list = CompanySymbolList::new();

        // Use a cursor to simulate a file reader from the string
        let mut reader = ReaderBuilder::new()
            .has_headers(true) // Ensure headers are expected
            .from_reader(Cursor::new(csv_str));

        // Extract column headers
        let headers = reader
            .headers()
            .map_err(|e| Error::ParserError(format!("Failed to read headers: {}", e)))?
            .clone();

        for record in reader.records() {
            let record =
                record.map_err(|e| Error::ParserError(format!("Failed to read record: {}", e)))?;

            // Extract values based on header names
            let symbol = record
                .get(headers.iter().position(|h| h == "Symbol").unwrap())
                .ok_or_else(|| Error::ParserError("Missing 'Symbol' field".to_string()))?;

            let company_name = record
                .get(headers.iter().position(|h| h == "Company Name").unwrap())
                .map(|name| name.to_string());

            let comma_separated_alternate_names =
                record.get(headers.iter().position(|h| h == "Alternate Names").unwrap());

            let alternate_names: Vec<String> = if let Some(names) = comma_separated_alternate_names
            {
                names
                    .split(',')
                    .map(|name| name.trim().to_string()) // Trim whitespace and convert to String
                    .collect()
            } else {
                Vec::new() // Default to an empty vector if alternate names are missing
            };

            company_symbols_list.push((symbol.to_uppercase(), company_name, alternate_names));
        }

        Ok(company_symbols_list)
    }
}
